use crate::arch::processor::features::enable_features;
use crate::arch::x86::kernel::descriptors::global;
use crate::arch::x86::kernel::interrupts;
use crate::arch::x86::kernel::devices::timer;

#[cfg(feature = "vga")]
use crate::arch::x86::kernel::devices::vga;

pub(crate) fn perform_early_initialization() {
    enable_features();
    global::init();
    interrupts::hardware::init();
    timer::initialize_programmable_interval_timer();

    #[cfg(feature = "vga")]
    vga::init();
}