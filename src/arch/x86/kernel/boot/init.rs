use {
    crate::{arch::{
        processor::features::enable_features,
        x86::{
            kernel::{interrupts, devices::timer, descriptors::global},
        },
    }},
};

#[cfg(feature = "vga")]
use crate::arch::x86::kernel::devices::vga;

pub(crate) fn early_init() {
    enable_features();
    global::init();
    interrupts::hardware::init();
    timer::init_timer();

    #[cfg(feature = "vga")]
    vga::init();
}