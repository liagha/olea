#[cfg(not(feature = "vga"))]
use crate::arch::serial;
#[cfg(feature = "vga")]
use crate::arch::vga;

use {
	super::{
		error::Error,
		descriptor::Interface,
	}
};

#[derive(Debug)]
pub struct GenericStandardInput;

impl Interface for GenericStandardInput {}

impl GenericStandardInput {
	pub const fn new() -> Self {
		Self {}
	}
}

#[derive(Debug)]
pub struct GenericStandardOutput;

impl Interface for GenericStandardOutput {
	fn write(&self, buf: &[u8]) -> Result<usize, Error> {
		cfg_if::cfg_if! {
            if #[cfg(feature = "vga")] {
                vga::VGA_SCREEN.lock().write_bytes(buf);
            } else {
                serial::PORT.lock().write_bytes(buf);
            }
        }
		Ok(buf.len())
	}
}

impl GenericStandardOutput {
	pub const fn new() -> Self {
		Self {}
	}
}

#[derive(Debug)]
pub struct GenericStandardError;

impl Interface for GenericStandardError {
	fn write(&self, buf: &[u8]) -> Result<usize, Error> {
		cfg_if::cfg_if! {
            if #[cfg(feature = "vga")] {
                vga::VGA_SCREEN.lock().write_bytes(buf);
            } else {
                serial::PORT.lock().write_bytes(buf);
            }
        }
		Ok(buf.len())
	}
}

impl GenericStandardError {
	pub const fn new() -> Self {
		Self {}
	}
}