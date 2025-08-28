#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate olea;
extern crate alloc;

use {
	alloc::string::String,
	olea::{
		io::{load_file, process_elf},
		scheduler::{self, task::NORMAL_PRIORITY},
		arch::{
			kernel::interrupts::interrupt_enable,
		},
	},
};

extern "C" fn create_user() {

	let path = String::from("/bin/demo");

	info!("started application loader.");

	let buffer = load_file(&path).unwrap();
	
	_ = process_elf(buffer);
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
	extern "C" fn task_test() {}

	println!("\n-     O L E A     -\n");

	olea::initialize();

	for _ in 0..2 {
		scheduler::spawn(task_test, NORMAL_PRIORITY).unwrap();
	}

	scheduler::spawn(create_user, NORMAL_PRIORITY).unwrap();

	interrupt_enable();

	scheduler::reschedule();

	info!("shutdown system.");

	0
}
