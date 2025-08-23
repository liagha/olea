#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate olea;
extern crate alloc;

use {
	alloc::string::String,
	olea::{
		arch::{self, load_application},
		scheduler::{self, task::NORMAL_PRIORITY},
		logging::{LogLevel, LOGGER},
	},
};
use olea::arch::x86::kernel::interrupts::hardware::irq_enable;

extern "C" fn create_user() {
	let path = String::from("/bin/demo");

	info!("started application loader.");

	_ = load_application(&path);
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
	extern "C" fn task_test() {}

	println!("\n-     O L E A     -\n");

	olea::init();

	for _ in 0..2 {
		scheduler::spawn(task_test, NORMAL_PRIORITY).unwrap();
	}

	scheduler::spawn(create_user, NORMAL_PRIORITY).unwrap();

	irq_enable();

	scheduler::reschedule();

	info!("shutdown system.");

	0
}
