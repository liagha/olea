// src/arch/x86/kernel/devices/timer.rs
use {
    crate::{
        consts::*,
    },
    x86::{
        io::*,
        time::rdtsc,
    },
};
use crate::arch::processor::utilities::memory_barrier;

const CLOCK_TICK_RATE: u32 = 1193182u32;

unsafe fn wait_some_time() {
    let start = rdtsc();

    memory_barrier();
    while rdtsc() - start < 1000000 {
        memory_barrier();
    }
}

pub(crate) fn initialize_programmable_interval_timer() {
    let latch = ((CLOCK_TICK_RATE + TIMER_FREQ / 2) / TIMER_FREQ) as u16;

    unsafe {
        outb(0x43, 0x34);

        wait_some_time();

        outb(0x40, (latch & 0xFF) as u8);

        wait_some_time();

        outb(0x40, (latch >> 8) as u8);
    }
}