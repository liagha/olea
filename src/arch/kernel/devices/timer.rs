use {
    crate::{
        consts::*,
        arch::kernel::processor::memory_barrier,
    },
    x86::{
        io::*,
        time::rdtsc,
    },
};

const CLOCK_TICK_RATE: u32 = 1193182u32;

unsafe fn wait_some_time() {
    let start = rdtsc();

    memory_barrier();
    while rdtsc() - start < 1000000 {
        memory_barrier();
    }
}

pub fn init() {
    let latch = ((CLOCK_TICK_RATE + TIMER_FREQ / 2) / TIMER_FREQ) as u16;

    unsafe {
        outb(0x43, 0x34);

        wait_some_time();

        outb(0x40, (latch & 0xFF) as u8);

        wait_some_time();

        outb(0x40, (latch >> 8) as u8);
    }
}