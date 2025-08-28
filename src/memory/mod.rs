pub mod freelist;

use crate::arch;
use crate::arch::memory::get_memory_size;
use crate::arch::kernel::processor::shutdown::shutdown;
pub mod buddy;
pub mod linked_list;

#[cfg(not(test))]
use alloc::alloc::Layout;

pub fn initialize() {
	info!("memory size: {} MB.", get_memory_size() >> 20);

	arch::memory::initialize();
}

#[cfg(not(test))]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
	println!(
		"out of memory: memory allocation of {} bytes failed.",
		layout.size()
	);

	shutdown(1);
}
