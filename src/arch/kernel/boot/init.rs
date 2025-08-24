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

pub fn early_init() {
    enable_features();
    global::init();
    interrupts::init();
    timer::init();

    #[cfg(feature = "vga")]
    vga::init();
}