#![allow(dead_code)]

pub mod boot;
pub mod calls;
pub mod descriptors;
pub mod devices;
pub mod interrupts;
pub mod processor;
pub mod scheduling;

use {
	bootloader::BootInfo,
	core::arch::asm,
};

#[cfg(target_arch = "x86_64")]
pub static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub fn register_task() {
	let sel: u16 = 6u16 << 3;

	unsafe {
		asm!("ltr ax", in("ax") sel, options(nostack, nomem));
	}
}

pub fn init() {
	boot::early_init();
}