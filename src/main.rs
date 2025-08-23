#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate olea;
extern crate alloc;

use alloc::string::String;
use olea::arch;
use olea::arch::load_application;
use olea::scheduler;
use olea::scheduler::task::NORMAL_PRIORITY;
use olea::{LogLevel, LOGGER};

extern "C" fn create_user_foo() {
	let path = String::from("/bin/demo");

	info!("Hello from loader");

	if load_application(&path).is_err() {
		error!("Unable to load elf64 binary {}", path)
	}
}

extern "C" fn foo() {
	println!("hello from task {}", scheduler::get_current_taskid());
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
	olea::init();

	println!("Hello from eduOS-rs!");

	for _i in 0..2 {
		scheduler::spawn(foo, NORMAL_PRIORITY).unwrap();
	}
	scheduler::spawn(create_user_foo, NORMAL_PRIORITY).unwrap();

	arch::irq::irq_enable();

	scheduler::reschedule();

	println!("Shutdown system!");

	0
}
