#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate olea;
extern crate alloc;

use alloc::vec::Vec;
use {
	alloc::string::String,
	olea::{
		arch::{
			process_elf,
			kernel::interrupts::interrupt_enable,
		},
		scheduler::{self, task::NORMAL_PRIORITY},
	},
};
use olea::io;

extern "C" fn create_user() {
	pub fn load_file(path: &String) -> Result<Vec<u8>, io::Error> {
		use olea::io::Read;

		debug!("attempting to load application from path.");

		let mut file = olea::file::File::open(path).map_err(|_| io::Error::FsError)?;
		let length = file.len().map_err(|_| io::Error::FsError)?;

		if length == 0 {
			error!("file is empty.");
			return Err(io::Error::InvalidArgument);
		}

		if length > usize::MAX {
			error!("file size exceeds maximum supported size.");
			return Err(io::Error::ValueOverflow);
		}

		debug!("file size is {} bytes.", length);
		let mut buffer: Vec<u8> = Vec::new();

		buffer.resize(length, 0);
		file.read(&mut buffer)?;

		drop(file);
		Ok(buffer)
	}

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

	olea::init();

	for _ in 0..2 {
		scheduler::spawn(task_test, NORMAL_PRIORITY).unwrap();
	}

	scheduler::spawn(create_user, NORMAL_PRIORITY).unwrap();

	interrupt_enable();

	scheduler::reschedule();

	info!("shutdown system.");

	0
}
