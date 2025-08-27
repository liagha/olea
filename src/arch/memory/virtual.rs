use crate::{
    memory::freelist::{FreeList, FreeListEntry},
    scheduler::DisabledPreemption,
};
use crate::arch::memory::{
    paging::{BasePageSize, PageSize},
    VirtualAddress,
};

static mut KERNEL_FREE_LIST: FreeList<VirtualAddress> = FreeList::new();

pub const KERNEL_VIRTUAL_MEMORY_START: VirtualAddress = VirtualAddress(0x8000_0000u64);

pub const KERNEL_VIRTUAL_MEMORY_END: VirtualAddress = VirtualAddress(0x800_0000_0000u64);

const TASK_VIRTUAL_MEMORY_END: VirtualAddress = VirtualAddress(0x8000_0000_0000u64);

pub fn init() {
	let entry = FreeListEntry {
		start: KERNEL_VIRTUAL_MEMORY_START,
		end: KERNEL_VIRTUAL_MEMORY_END,
	};
	unsafe {
		KERNEL_FREE_LIST.list.push_back(entry);
	}
}

#[allow(dead_code)]
pub fn allocate(size: usize) -> VirtualAddress {
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "size `{:#X}` is not a multiple of `{:#X}`.", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { KERNEL_FREE_LIST.allocate(size, None) };
	assert!(
		result.is_ok(),
		"could not allocate `{:#X}` bytes of virtual memory.",
		size
	);
	result.unwrap()
}

pub fn allocate_aligned(size: usize, alignment: usize) -> VirtualAddress {
	assert!(size > 0);
	assert!(alignment > 0);
	assert_eq!(size % alignment, 0, "size `{:#X}` is not a multiple of the given alignment `{:#X}`.", size, alignment);
	assert_eq!(alignment % BasePageSize::SIZE, 0, "alignment `{:#X}` is not a multiple of `{:#X}`.", alignment, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { KERNEL_FREE_LIST.allocate(size, Some(alignment)) };
	assert!(
		result.is_ok(),
		"could not allocate `{:#X}` bytes of virtual memory aligned to `{}` bytes.",
		size,
		alignment
	);
	result.unwrap()
}

pub fn deallocate(virtual_address: VirtualAddress, size: usize) {
	assert!(
		virtual_address < KERNEL_VIRTUAL_MEMORY_END,
		"virtual address `{:#X}` is not smaller than `KERNEL_VIRTUAL_MEMORY_END`.",
		virtual_address
	);
	assert_eq!(virtual_address % BasePageSize::SIZE, 0, "virtual address `{:#X}` is not a multiple of `{:#X}`.", virtual_address, BasePageSize::SIZE);
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "size `{:#X}` is not a multiple of `{:#X}`.", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	unsafe {
		KERNEL_FREE_LIST.deallocate(virtual_address, size);
	}
}

#[allow(dead_code)]
pub fn task_heap_start() -> VirtualAddress {
	KERNEL_VIRTUAL_MEMORY_END
}

#[allow(dead_code)]
pub fn task_heap_end() -> VirtualAddress {
	TASK_VIRTUAL_MEMORY_END
}
