use {
	crate::{
		arch::x86::memory::{
			VirtAddr,
			paging::{BasePageSize, PageSize},
		},
		memory::freelist::{FreeList, FreeListEntry},
		scheduler::DisabledPreemption,
	}
};

static mut KERNEL_FREE_LIST: FreeList<VirtAddr> = FreeList::new();

pub(crate) const KERNEL_VIRTUAL_MEMORY_START: VirtAddr = VirtAddr(0x8000_0000u64);

pub(crate) const KERNEL_VIRTUAL_MEMORY_END: VirtAddr = VirtAddr(0x800_0000_0000u64);

const TASK_VIRTUAL_MEMORY_END: VirtAddr = VirtAddr(0x8000_0000_0000u64);

pub(crate) fn init() {
	let entry = FreeListEntry {
		start: KERNEL_VIRTUAL_MEMORY_START,
		end: KERNEL_VIRTUAL_MEMORY_END,
	};
	unsafe {
		KERNEL_FREE_LIST.list.push_back(entry);
	}
}

#[allow(dead_code)]
pub(crate) fn allocate(size: usize) -> VirtAddr {
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { KERNEL_FREE_LIST.allocate(size, None) };
	assert!(
		result.is_ok(),
		"Could not allocate {:#X} bytes of virtual memory",
		size
	);
	result.unwrap()
}

pub(crate) fn allocate_aligned(size: usize, alignment: usize) -> VirtAddr {
	assert!(size > 0);
	assert!(alignment > 0);
	assert_eq!(size % alignment, 0, "Size {:#X} is not a multiple of the given alignment {:#X}", size, alignment);
	assert_eq!(alignment % BasePageSize::SIZE, 0, "Alignment {:#X} is not a multiple of {:#X}", alignment, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	let result = unsafe { KERNEL_FREE_LIST.allocate(size, Some(alignment)) };
	assert!(
		result.is_ok(),
		"Could not allocate {:#X} bytes of virtual memory aligned to {} bytes",
		size,
		alignment
	);
	result.unwrap()
}

pub(crate) fn deallocate(virtual_address: VirtAddr, size: usize) {
	assert!(
		virtual_address < KERNEL_VIRTUAL_MEMORY_END,
		"Virtual address {:#X} is not < KERNEL_VIRTUAL_MEMORY_END",
		virtual_address
	);
	assert_eq!(virtual_address % BasePageSize::SIZE, 0, "Virtual address {:#X} is not a multiple of {:#X}", virtual_address, BasePageSize::SIZE);
	assert!(size > 0);
	assert_eq!(size % BasePageSize::SIZE, 0, "Size {:#X} is not a multiple of {:#X}", size, BasePageSize::SIZE);

	let _preemption = DisabledPreemption::new();
	unsafe {
		KERNEL_FREE_LIST.deallocate(virtual_address, size);
	}
}

#[allow(dead_code)]
pub(crate) fn task_heap_start() -> VirtAddr {
	KERNEL_VIRTUAL_MEMORY_END
}

#[allow(dead_code)]
pub(crate) fn task_heap_end() -> VirtAddr {
	TASK_VIRTUAL_MEMORY_END
}
