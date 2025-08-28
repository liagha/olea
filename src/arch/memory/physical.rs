use {
    crate::{
        memory::freelist::{FreeList, FreeListEntry},
        scheduler::DisabledPreemption,
    },
    core::ops::Deref,
};
use crate::arch::kernel::BOOT_INFO;
use crate::arch::memory::{
    paging::{BasePageSize, PageSize},
    PhysicalAddress,
};

static mut PHYSICAL_FREE_LIST: FreeList<PhysicalAddress> = FreeList::new();

pub fn initialize() {
	unsafe {
		let regions = BOOT_INFO.unwrap().memory_map.deref();

		for i in regions {
			if i.region_type == bootloader::bootinfo::MemoryRegionType::Usable {
				let entry = FreeListEntry {
					start: (i.range.start_frame_number * 0x1000).into(),
					end: (i.range.end_frame_number * 0x1000).into(),
				};

				debug!("add free physical regions 0x{:x} - 0x{:x}.", entry.start, entry.end);
				PHYSICAL_FREE_LIST.list.push_back(entry);
			}
		}
	}
}

pub fn allocate(size: usize) -> PhysicalAddress {
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "size `{:#X}` is not a multiple of `{:#X}`.", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size, None) };
	assert!(result.is_ok(), "could not allocate `{:#X}` bytes of physical memory.", size);
	result.unwrap()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> PhysicalAddress {
	assert!(size > 0);
	assert!(alignment > 0);
	assert_eq!(size % alignment, 0, "size `{:#X}` is not a multiple of the given alignment `{:#X}`.", size, alignment);
	assert_eq!(alignment % BasePageSize::SIZE, 0, "alignment `{:#X}` is not a multiple of `{:#X}`.", alignment, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { PHYSICAL_FREE_LIST.allocate(size, Some(alignment)) };
	assert!(result.is_ok(), "could not allocate `{:#X}` bytes of physical memory aligned to `{}` bytes.", size, alignment);
	result.unwrap()
}

pub fn deallocate(physical_address: PhysicalAddress, size: usize) {
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "size `{:#X}` is not a multiple of `{:#X}`.", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	unsafe {
		PHYSICAL_FREE_LIST.deallocate(physical_address, size);
	}
}