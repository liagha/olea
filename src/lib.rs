#![feature(const_mut_refs)]
#![feature(linked_list_cursors)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(specialization)]
#![feature(map_try_insert)]
#![allow(clippy::module_inception)]
#![allow(incomplete_features)]
#![allow(static_mut_refs)]
#![no_std]

extern crate alloc;
#[cfg(target_arch = "x86_64")]
extern crate x86;
#[macro_use]
extern crate bitflags;
extern crate num_traits;

use {
	crate::{
		arch::processor::shutdown,
		consts::HEAP_SIZE,
		memory::buddy::LockedHeap,
	},
	core::panic::PanicInfo,
};

#[macro_use]
pub mod macros;
#[macro_use]
pub mod logging;
pub mod arch;
pub mod collections;
pub mod console;
pub mod consts;
pub mod errno;
pub mod file;
pub mod io;
pub mod memory;
pub mod scheduler;
pub mod sync;
pub mod call;

#[repr(align(256))]
struct Arena([u8; HEAP_SIZE]);

impl Arena {
	pub const fn new() -> Self {
		Self([0; HEAP_SIZE])
	}
}

static mut ARENA: Arena = Arena::new();

#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();

pub fn init() {
	unsafe {
		ALLOCATOR.init(ARENA.0.as_mut_ptr(), HEAP_SIZE);
	}

	arch::init();
	memory::init();
	scheduler::init();
	file::init();
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
	print!("panic: ");

	if let Some(location) = info.location() {
		print!("{}:{}: ", location.file(), location.line());
	}

	if let Some(message) = info.message().as_str() {
		print!("{}", message);
	}

	print!("\n");

	shutdown::shutdown(1);
}
