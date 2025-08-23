#[cfg(not(feature = "vga"))]
use crate::arch::serial;
#[cfg(feature = "vga")]
use crate::arch::vga;

use core::fmt;

pub struct Console;

impl fmt::Write for Console {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		cfg_if::cfg_if! {
			if #[cfg(feature = "vga")] {
				vga::VGA_SCREEN.lock().write_str(s)
			} else {
				serial::COM1.lock().write_str(s)
			}
		}
	}
}
