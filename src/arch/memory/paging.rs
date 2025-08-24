use {
    crate::{
        arch::kernel::processor::features::{get_physical_address_bits, supports_1gib_pages},
        consts::*,
        logging::*,
        scheduler,
    },
    core::{
        arch::asm,
        convert::TryInto,
        marker::PhantomData,
        mem::size_of,
        ptr::write_bytes,
    },
    x86::{
        controlregs,
        irq::*,
    },
};
use crate::arch::kernel::{
    interrupts::{
        exceptions::ExceptionStackFrame,
        hardware::{end_of_interrupt, interrupt_nested_disable, interrupt_nested_enable, MASTER},
    },
    BOOT_INFO,
};
use crate::arch::memory::{
    physical, r#virtual,
    PhysAddr, VirtAddr,
};

const PML4_ADDRESS: *mut PageTable<PML4> = 0xFFFF_FFFF_FFFF_F000 as *mut PageTable<PML4>;
const PAGE_BITS: usize = 12;
const PAGE_MAP_BITS: usize = 9;
const PAGE_MAP_MASK: usize = 0x1FF;

bitflags! {
	#[derive(Debug, Copy, Clone)]
	pub struct PageTableEntryFlags: usize {
		const PRESENT = 1 << 0;
		const WRITABLE = 1 << 1;
		const USER_ACCESSIBLE = 1 << 2;
		const WRITE_THROUGH = 1 << 3;
		const CACHE_DISABLE = 1 << 4;
		const ACCESSED = 1 << 5;
		const DIRTY = 1 << 6;
		const HUGE_PAGE = 1 << 7;
		const GLOBAL = 1 << 8;
		#[cfg(target_arch = "x86_64")]
		const EXECUTE_DISABLE = 1 << 63;
	}
}

impl PageTableEntryFlags {
	const BLANK: PageTableEntryFlags = PageTableEntryFlags::empty();

	pub fn device(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::CACHE_DISABLE);
		self
	}

	pub fn normal(&mut self) -> &mut Self {
		self.remove(PageTableEntryFlags::CACHE_DISABLE);
		self
	}

	pub fn read_only(&mut self) -> &mut Self {
		self.remove(PageTableEntryFlags::WRITABLE);
		self
	}

	pub fn writable(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::WRITABLE);
		self
	}

	pub fn execute_disable(&mut self) -> &mut Self {
		self.insert(PageTableEntryFlags::EXECUTE_DISABLE);
		self
	}
}

#[derive(Clone, Copy)]
pub struct PageTableEntry {
	physical_address_and_flags: PhysAddr,
}

impl PageTableEntry {
	pub fn address(&self) -> PhysAddr {
		PhysAddr::from(
			self.physical_address_and_flags.as_usize()
				& !(BasePageSize::SIZE - 1)
				& !PageTableEntryFlags::EXECUTE_DISABLE.bits(),
		)
	}

	fn is_present(&self) -> bool {
		(self.physical_address_and_flags.as_usize() & PageTableEntryFlags::PRESENT.bits()) != 0
	}

	#[allow(dead_code)]
	fn is_huge(&self) -> bool {
		(self.physical_address_and_flags.as_usize() & PageTableEntryFlags::HUGE_PAGE.bits()) != 0
	}

	fn is_user(&self) -> bool {
		(self.physical_address_and_flags.as_usize() & PageTableEntryFlags::USER_ACCESSIBLE.bits()) != 0
	}

	fn set(&mut self, physical_address: PhysAddr, flags: PageTableEntryFlags) {
		if flags.contains(PageTableEntryFlags::HUGE_PAGE) {
			assert_eq!(physical_address % LargePageSize::SIZE, 0, "physical address is not on a `2 MB` page boundary (physical_address = `{:#X}`).", physical_address);
		} else {
			assert_eq!(physical_address % BasePageSize::SIZE, 0, "physical address is not on a `4 KB` page boundary (physical_address = `{:#X}`).", physical_address);
		}

		assert_eq!(
			physical_address.as_u64().checked_shr(get_physical_address_bits() as u32),
			Some(0),
			"physical address exceeds CPU's physical address width (physical_address = `{:#X}`).",
			physical_address
		);

		let mut flags_to_set = flags;
		flags_to_set.insert(PageTableEntryFlags::PRESENT | PageTableEntryFlags::ACCESSED);
		self.physical_address_and_flags = PhysAddr::from(physical_address.as_usize() | flags_to_set.bits());
	}
}

pub trait PageSize: Copy {
	const SIZE: usize;
	const MAP_LEVEL: usize;
	const MAP_EXTRA_FLAG: PageTableEntryFlags;
}

#[derive(Clone, Copy)]
pub enum BasePageSize {}
impl PageSize for BasePageSize {
	const SIZE: usize = 0x1000;
	const MAP_LEVEL: usize = 0;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

#[derive(Clone, Copy)]
pub enum LargePageSize {}
impl PageSize for LargePageSize {
	const SIZE: usize = 0x200000;
	const MAP_LEVEL: usize = 1;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::HUGE_PAGE;
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum HugePageSize {}
impl PageSize for HugePageSize {
	const SIZE: usize = 0x40000000;
	const MAP_LEVEL: usize = 2;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::HUGE_PAGE;
}

#[derive(Clone, Copy)]
struct Page<S: PageSize> {
	virtual_address: VirtAddr,
	size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
	#[allow(dead_code)]
	fn address(&self) -> VirtAddr {
		self.virtual_address
	}

	#[inline(always)]
	fn flush_from_tlb(&self) {
		unsafe {
			asm!("invlpg [{}]", in(reg) self.virtual_address.as_u64(), options(preserves_flags, nostack));
		}
	}

	fn is_valid_address(virtual_address: VirtAddr) -> bool {
		virtual_address < VirtAddr(0x8000_0000_0000u64)
			|| virtual_address >= VirtAddr(0xFFFF_8000_0000_0000u64)
	}

	fn including_address(virtual_address: VirtAddr) -> Self {
		assert!(
			Self::is_valid_address(virtual_address),
			"virtual address `{:#X}` is invalid.",
			virtual_address
		);

		if S::SIZE == 1024 * 1024 * 1024 {
			assert!(supports_1gib_pages());
		}

		Self {
			virtual_address: align_down!(virtual_address, S::SIZE),
			size: PhantomData,
		}
	}

	fn range(first: Self, last: Self) -> PageIter<S> {
		assert!(first.virtual_address <= last.virtual_address);
		PageIter { current: first, last }
	}

	fn table_index<L: PageTableLevel>(&self) -> usize {
		assert!(L::LEVEL >= S::MAP_LEVEL);
		self.virtual_address.as_usize() >> PAGE_BITS >> (L::LEVEL * PAGE_MAP_BITS) & PAGE_MAP_MASK
	}
}

struct PageIter<S: PageSize> {
	current: Page<S>,
	last: Page<S>,
}

impl<S: PageSize> Iterator for PageIter<S> {
	type Item = Page<S>;

	fn next(&mut self) -> Option<Page<S>> {
		if self.current.virtual_address <= self.last.virtual_address {
			let p = self.current;
			self.current.virtual_address += S::SIZE;
			Some(p)
		} else {
			None
		}
	}
}

trait PageTableLevel {
	const LEVEL: usize;
}

trait PageTableLevelWithSubtables: PageTableLevel {
	type SubtableLevel;
}

enum PML4 {}
impl PageTableLevel for PML4 {
	const LEVEL: usize = 3;
}
impl PageTableLevelWithSubtables for PML4 {
	type SubtableLevel = PDPT;
}

enum PDPT {}
impl PageTableLevel for PDPT {
	const LEVEL: usize = 2;
}
impl PageTableLevelWithSubtables for PDPT {
	type SubtableLevel = PD;
}

enum PD {}
impl PageTableLevel for PD {
	const LEVEL: usize = 1;
}
impl PageTableLevelWithSubtables for PD {
	type SubtableLevel = PT;
}

enum PT {}
impl PageTableLevel for PT {
	const LEVEL: usize = 0;
}

struct PageTable<L> {
	entries: [PageTableEntry; 1 << PAGE_MAP_BITS],
	level: PhantomData<L>,
}

trait PageTableMethods {
	fn get_page_table_entry<S: PageSize>(&mut self, page: Page<S>) -> Option<PageTableEntry>;
	fn map_page_in_this_table<S: PageSize>(&mut self, page: Page<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) -> bool;
	fn map_page<S: PageSize>(&mut self, page: Page<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) -> bool;
	fn drop_user_space(&mut self);
}

impl<L: PageTableLevel> PageTableMethods for PageTable<L> {
	default fn get_page_table_entry<S: PageSize>(&mut self, page: Page<S>) -> Option<PageTableEntry> {
		assert_eq!(L::LEVEL, S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			Some(self.entries[index])
		} else {
			None
		}
	}

	fn map_page_in_this_table<S: PageSize>(&mut self, page: Page<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) -> bool {
		assert_eq!(L::LEVEL, S::MAP_LEVEL);
		let index = page.table_index::<L>();
		let flush = self.entries[index].is_present();

		self.entries[index].set(physical_address, PageTableEntryFlags::DIRTY | S::MAP_EXTRA_FLAG | flags);

		if flush {
			page.flush_from_tlb();
		}

		flush
	}

	default fn map_page<S: PageSize>(&mut self, page: Page<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) -> bool {
		self.map_page_in_this_table::<S>(page, physical_address, flags)
	}

	default fn drop_user_space(&mut self) {
		let last = 1 << PAGE_MAP_BITS;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				let physical_address = self.entries[index].address();
				debug!("free page frame at 0x{:x}.", physical_address);
				physical::deallocate(physical_address, BasePageSize::SIZE);
			}
		}
	}
}

impl<L: PageTableLevelWithSubtables> PageTableMethods for PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	fn get_page_table_entry<S: PageSize>(&mut self, page: Page<S>) -> Option<PageTableEntry> {
		assert!(L::LEVEL >= S::MAP_LEVEL);
		let index = page.table_index::<L>();

		if self.entries[index].is_present() {
			if L::LEVEL > S::MAP_LEVEL {
				let subtable = self.subtable::<S>(page);
				subtable.get_page_table_entry::<S>(page)
			} else {
				Some(self.entries[index])
			}
		} else {
			None
		}
	}

	fn map_page<S: PageSize>(&mut self, page: Page<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) -> bool {
		assert!(L::LEVEL >= S::MAP_LEVEL);

		if L::LEVEL > S::MAP_LEVEL {
			let index = page.table_index::<L>();

			if !self.entries[index].is_present() {
				let pt_addr = physical::allocate(BasePageSize::SIZE);
				let entry_flags = if flags.contains(PageTableEntryFlags::USER_ACCESSIBLE) {
					PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE
				} else {
					PageTableEntryFlags::WRITABLE
				};
				self.entries[index].set(pt_addr, entry_flags);

				let subtable = self.subtable::<S>(page);
				for entry in subtable.entries.iter_mut() {
					entry.physical_address_and_flags = PhysAddr::zero();
				}

				subtable.map_page::<S>(page, physical_address, flags)
			} else {
				let subtable = self.subtable::<S>(page);
				subtable.map_page::<S>(page, physical_address, flags)
			}
		} else {
			self.map_page_in_this_table::<S>(page, physical_address, flags)
		}
	}

	fn drop_user_space(&mut self) {
		let last = 1 << PAGE_MAP_BITS;
		let table_address = self as *const PageTable<L> as usize;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				if L::LEVEL > BasePageSize::MAP_LEVEL {
					let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
					let subtable = unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) };
					subtable.drop_user_space();
				}
			}
		}
	}
}

impl<L: PageTableLevelWithSubtables> PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	fn subtable<S: PageSize>(&mut self, page: Page<S>) -> &mut PageTable<L::SubtableLevel> {
		assert!(L::LEVEL > S::MAP_LEVEL);

		let index = page.table_index::<L>();
		let table_address = self as *const PageTable<L> as usize;
		let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
		unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) }
	}

	fn map_pages<S: PageSize>(&mut self, range: PageIter<S>, physical_address: PhysAddr, flags: PageTableEntryFlags) {
		let mut current_physical_address = physical_address;

		for page in range {
			self.map_page(page, current_physical_address, flags);
			current_physical_address += S::SIZE as u64;
		}
	}

	fn drop_user_space(&mut self) {
		assert_eq!(L::LEVEL, PML4::LEVEL);

		let last = (1 << PAGE_MAP_BITS) - 1;
		let table_address = self as *const PageTable<L> as usize;

		for index in 0..last {
			if self.entries[index].is_present() && self.entries[index].is_user() {
				let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
				let subtable = unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) };

				subtable.drop_user_space();

				let physical_address = self.entries[index].address();
				debug!("free page table at 0x{:x}.", physical_address);
				physical::deallocate(physical_address, BasePageSize::SIZE);
			}
		}
	}
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: ExceptionStackFrame, error_code: u64) {
	let mut virtual_address = unsafe { VirtAddr::from_usize(controlregs::cr2()) };

	if virtual_address > USER_ENTRY + 0x400000u64 - 64u64 * 1024u64 {
		virtual_address = align_down!(virtual_address, BasePageSize::SIZE);

		let physical_address = physical::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);

		debug!("map 0x{:x} into the user space at 0x{:x}.", physical_address, virtual_address);

		map::<BasePageSize>(
			virtual_address,
			physical_address,
			1,
			PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE | PageTableEntryFlags::EXECUTE_DISABLE,
		);

		unsafe {
			write_bytes(virtual_address.as_mut_ptr::<u8>(), 0x00, BasePageSize::SIZE);
			controlregs::cr2_write(0);
		}

		end_of_interrupt(MASTER);
	} else {
		let pferror = PageFaultError::from_bits_truncate(error_code as u32);

		error!("page fault (#PF) Exception: {:#?}.", stack_frame);
		error!("virtual_address = {:#X}, page fault error = {}.", virtual_address, pferror);

		unsafe {
			controlregs::cr2_write(0);
		}

		end_of_interrupt(MASTER);
		scheduler::abort();
	}
}

fn get_page_range<S: PageSize>(virtual_address: VirtAddr, count: usize) -> PageIter<S> {
	let first_page = Page::<S>::including_address(virtual_address);
	let last_page = Page::<S>::including_address(virtual_address + (count - 1) * S::SIZE);
	Page::range(first_page, last_page)
}

#[allow(dead_code)]
pub fn get_page_table_entry<S: PageSize>(virtual_address: VirtAddr) -> Option<PageTableEntry> {
	debug!("looking up Page Table Entry for {:#X}.", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.get_page_table_entry(page)
}

pub fn get_physical_address<S: PageSize>(virtual_address: VirtAddr) -> PhysAddr {
	debug!("getting physical address for {:#X}.", virtual_address);

	let page = Page::<S>::including_address(virtual_address);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	let address = root_pagetable
		.get_page_table_entry(page)
		.expect("Entry not present")
		.address();
	let offset = virtual_address & (S::SIZE - 1);
	address | PhysAddr(offset.as_u64())
}

pub fn virtual_to_physical(virtual_address: VirtAddr) -> PhysAddr {
	get_physical_address::<BasePageSize>(virtual_address)
}

pub fn unmap<S: PageSize>(virtual_address: VirtAddr, count: usize) {
	debug!("unmapping virtual address {:#X} ({} pages).", virtual_address, count);

	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.map_pages(range, PhysAddr::zero(), PageTableEntryFlags::BLANK);
}

pub fn map<S: PageSize>(virtual_address: VirtAddr, physical_address: PhysAddr, count: usize, flags: PageTableEntryFlags) {
	debug!("mapping virtual address {:#X} to physical address {:#X} ({} pages).", virtual_address, physical_address, count);

	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.map_pages(range, physical_address, flags);
}

static mut ROOT_PAGE_TABLE: PhysAddr = PhysAddr::zero();

#[inline(always)]
pub fn get_kernel_root_page_table() -> PhysAddr {
	unsafe { ROOT_PAGE_TABLE }
}

pub fn map_usr_entry(func: extern "C" fn()) {
	let irq = interrupt_nested_disable();

	let addr = VirtAddr::from_usize(align_down!((func as *const ()) as usize, BasePageSize::SIZE));

	map::<BasePageSize>(
		USER_ENTRY,
		virtual_to_physical(addr),
		2,
		PageTableEntryFlags::WRITABLE | PageTableEntryFlags::USER_ACCESSIBLE,
	);

	interrupt_nested_enable(irq);
}

pub fn drop_user_space() {
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.drop_user_space();
}

pub fn create_usr_pgd() -> PhysAddr {
	let irq = interrupt_nested_disable();

	debug!("create 1st level page table for the user-level task.");

	unsafe {
		let physical_address = physical::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);
		let user_page_table = r#virtual::allocate_aligned(BasePageSize::SIZE, BasePageSize::SIZE);

		debug!("map page frame 0x{:x} at virtual address 0x{:x}.", physical_address, user_page_table);

		map::<BasePageSize>(
			user_page_table,
			physical_address,
			1,
			PageTableEntryFlags::WRITABLE | PageTableEntryFlags::EXECUTE_DISABLE,
		);

		write_bytes(user_page_table.as_mut_ptr::<u8>(), 0x00, BasePageSize::SIZE);

		let recursive_pgt = BOOT_INFO.unwrap().recursive_page_table_addr as *const u64;
		let recursive_pgt_idx = BOOT_INFO.unwrap().recursive_index();
		let pml4 = user_page_table.as_mut_ptr::<u64>();
		for i in 0..recursive_pgt_idx + 2 {
			*pml4.offset(i.try_into().unwrap()) = *recursive_pgt.offset(i.try_into().unwrap());
		}

		let pml4 = (user_page_table.as_usize() + BasePageSize::SIZE - size_of::<usize>()) as *mut PageTableEntry;
		(*pml4).set(physical_address, PageTableEntryFlags::WRITABLE);

		unmap::<BasePageSize>(user_page_table, 1);
		r#virtual::deallocate(user_page_table, BasePageSize::SIZE);

		scheduler::set_root_page_table(physical_address);

		interrupt_nested_enable(irq);

		physical_address
	}
}

pub fn init() {
	let recursive_pgt = unsafe { BOOT_INFO.unwrap().recursive_page_table_addr } as *mut u64;
	let recursive_pgt_idx = unsafe { BOOT_INFO.unwrap().recursive_index() };

	debug!("found recursive_page_table_addr at 0x{:x}.", recursive_pgt as u64);
	debug!("recursive index: {}.", recursive_pgt_idx);

	unsafe {
		ROOT_PAGE_TABLE = PhysAddr::from(
			*recursive_pgt.offset(recursive_pgt_idx.try_into().unwrap()) as usize & !(BasePageSize::SIZE - 1),
		);
		*recursive_pgt.offset(511) = *recursive_pgt.offset(recursive_pgt_idx.try_into().unwrap());

		for i in recursive_pgt_idx + 2..511 {
			*recursive_pgt.offset(i.try_into().unwrap()) = 0;
		}

		controlregs::cr3_write(controlregs::cr3());
	}
}