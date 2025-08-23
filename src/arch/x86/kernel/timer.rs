use {
	crate::{
		arch::processor::*,
		consts::*,
		logging::*,
	},
	x86::{
		io::*,
		time::rdtsc,
	},
};

const CLOCK_TICK_RATE: u32 = 1193182u32;

unsafe fn wait_some_time() {
	let start = rdtsc();

	mb();
	while rdtsc() - start < 1000000 {
		mb();
	}
}

/// Initialize the Programmable Interrupt Controller (PIC)
pub(crate) fn init() {
	debug!("initialize timer");

	let latch = ((CLOCK_TICK_RATE + TIMER_FREQ / 2) / TIMER_FREQ) as u16;

	unsafe {
		/*
		 * Port 0x43 is for initializing the PIT:
		 *
		 * 0x34 means the following:
		 * 0b...     (step-by-step binary representation)
		 * ...  00  - channel 0
		 * ...  11  - write two values to counter register:
		 *            first low-, then high-byte
		 * ... 010  - mode number 2: "rate generator" / frequency divider
		 * ...   0  - binary counter (the alternative is BCD)
		*/

		outb(0x43, 0x34);

		wait_some_time();

		outb(0x40, (latch & 0xFF) as u8);

		wait_some_time();

		outb(0x40, (latch >> 8) as u8);
	}
}
