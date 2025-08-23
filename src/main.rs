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
		scheduler,
		scheduler::task::NORMAL_PRIORITY,
		logging::{LogLevel, LOGGER},
	},
};

extern "C" fn create_user_foo() {
	let path = String::from("/bin/demo");

	info!("Started Loader.");

	_ = load_application(&path);
}

extern "C" fn foo() {
	println!("Task {}.", scheduler::get_current_taskid());
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
	olea::init();

	println!("Olea-Base");

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(create_user_foo, NORMAL_PRIORITY).unwrap();

	arch::irq::irq_enable();

	scheduler::reschedule();

	println!("Shutdown system!");

	0
}
