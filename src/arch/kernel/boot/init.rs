use {
    crate::{
        arch::{
            kernel::{
                interrupts,
                descriptors::global, 
                devices::timer,
                processor::enable_features,
            },
        }
    }
};

#[cfg(feature = "vga")]
use crate::arch::kernel::devices::vga;

pub fn initialize() {
    enable_features();
    global::initialize();
    interrupts::initialize();
    timer::initialize();

    #[cfg(feature = "vga")]
    vga::initialize();
}