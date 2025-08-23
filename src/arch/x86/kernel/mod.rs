#![allow(dead_code)]

pub mod descriptors;
pub mod interrupts;
pub mod processor;
pub mod scheduling;
pub mod calls;
pub mod devices;
pub mod boot;

use bootloader::BootInfo;
use core::arch::asm;

#[cfg(target_arch = "x86_64")]
pub(crate) static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub fn register_task() {
	let sel: u16 = 6u16 << 3;

	unsafe {
		asm!("ltr ax", in("ax") sel, options(nostack, nomem));
	}
}

pub fn initialize() {
	boot::init::early_init();
}