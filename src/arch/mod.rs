pub mod kernel;
pub mod memory;
mod x86;

pub use {
    core::arch::{naked_asm, asm},
};

#[cfg(feature = "vga")]
pub use kernel::vga;

#[cfg(not(feature = "vga"))]
pub use kernel::devices::serial;
