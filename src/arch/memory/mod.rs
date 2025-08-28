pub mod paging;
pub mod physical;
pub mod r#virtual;

use {
    crate::{
        consts::INTERRUPT_STACK_SIZE,
        scheduler::task::Stack,
    },
    bootloader::bootinfo::MemoryRegionType,
    core::{
        convert::TryInto,
        ops::Deref,
    },
};

#[cfg(target_arch = "x86")]
pub use x86::bits32::paging::VAddr as VirtualAddress;
#[cfg(target_arch = "x86_64")]
pub use x86::bits64::paging::PAddr as PhysicalAddress;
#[cfg(target_arch = "x86_64")]
pub use x86::bits64::paging::VAddr as VirtualAddress;
use crate::arch::kernel::BOOT_INFO;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct BootStack {
	start: VirtualAddress,
	end: VirtualAddress,
	ist_start: VirtualAddress,
	ist_end: VirtualAddress,
}

impl BootStack {
	pub const fn new(
		start: VirtualAddress,
		end: VirtualAddress,
		ist_start: VirtualAddress,
		ist_end: VirtualAddress,
	) -> Self {
		Self {
			start,
			end,
			ist_start,
			ist_end,
		}
	}
}

impl Stack for BootStack {
	fn top(&self) -> VirtualAddress {
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "x86")] {
				self.end - 16u32
			} else {
				self.end - 16u64
			}
		}
	}

	fn bottom(&self) -> VirtualAddress {
		self.start
	}

	fn interrupt_top(&self) -> VirtualAddress {
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "x86")] {
				self.ist_end - 16u32
			} else {
				self.ist_end - 16u64
			}
		}
	}

	fn interrupt_bottom(&self) -> VirtualAddress {
		self.ist_start
	}
}

#[cfg(target_arch = "x86_64")]
pub fn get_boot_stack() -> BootStack {
	use crate::arch::kernel::BOOT_INFO;
	use bootloader::bootinfo::MemoryRegionType;
	use core::ops::Deref;

	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::KernelStack {
				return BootStack::new(
					VirtualAddress(i.range.start_frame_number * 0x1000),
					VirtualAddress(i.range.end_frame_number * 0x1000),
					VirtualAddress((BOOT_IST_STACK.0.as_ptr() as usize).try_into().unwrap()),
					VirtualAddress(
						(BOOT_IST_STACK.0.as_ptr() as usize + INTERRUPT_STACK_SIZE)
							.try_into()
							.unwrap(),
					),
				);
			}
		}

		panic!("unable to determine the kernel stack.");
	}
}

#[allow(dead_code)]
pub fn is_kernel(addr: VirtualAddress) -> bool {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == MemoryRegionType::Kernel
				&& addr >= VirtualAddress(i.range.start_frame_number * 0x1000)
				&& addr <= VirtualAddress(i.range.end_frame_number * 0x1000)
			{
				return true;
			}

			if i.region_type == MemoryRegionType::KernelStack
				&& addr >= VirtualAddress(i.range.start_frame_number * 0x1000)
				&& addr <= VirtualAddress(i.range.end_frame_number * 0x1000)
			{
				return true;
			}
		}
	}

	false
}

pub fn get_memory_size() -> usize {
	let mut sz: u64 = 0;

	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			match i.region_type {
				MemoryRegionType::Usable
				| MemoryRegionType::InUse
				| MemoryRegionType::Kernel
				| MemoryRegionType::KernelStack
				| MemoryRegionType::PageTable
				| MemoryRegionType::Bootloader
				| MemoryRegionType::FrameZero
				| MemoryRegionType::BootInfo
				| MemoryRegionType::Package => {
					sz += (i.range.end_frame_number - i.range.start_frame_number) * 0x1000
				}
				_ => {}
			}
		}
	}

	sz.try_into().unwrap()
}

pub fn initialize() {
	paging::initialize();
	physical::initialize();
	r#virtual::initialize();
}

#[repr(C, align(64))]
pub struct Aligned<T>(T);

impl<T> Aligned<T> {
	/// Constructor.
	pub const fn new(t: T) -> Self {
		Self(t)
	}
}

#[cfg(target_arch = "x86")]
pub const BOOT_STACK_SIZE: usize = 0x10000;
#[cfg(target_arch = "x86")]
#[link_section = ".data"]
pub static mut BOOT_STACK: Aligned<[u8; BOOT_STACK_SIZE]> =
	Aligned::new([0; BOOT_STACK_SIZE]);

pub static mut BOOT_IST_STACK: Aligned<[u8; INTERRUPT_STACK_SIZE]> =
	Aligned::new([0; INTERRUPT_STACK_SIZE]);

#[cfg(target_arch = "x86")]
pub fn get_boot_stack() -> BootStack {
	BootStack::new(
		unsafe { VirtualAddress((BOOT_STACK.0.as_ptr() as usize).try_into().unwrap()) },
		unsafe {
			VirtualAddress(
				(BOOT_STACK.0.as_ptr() as usize + BOOT_STACK_SIZE)
					.try_into()
					.unwrap(),
			)
		},
		unsafe { VirtualAddress((BOOT_IST_STACK.0.as_ptr() as usize).try_into().unwrap()) },
		unsafe {
			VirtualAddress(
				(BOOT_IST_STACK.0.as_ptr() as usize + INTERRUPT_STACK_SIZE)
					.try_into()
					.unwrap(),
			)
		},
	)
}
