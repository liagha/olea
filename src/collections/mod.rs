use crate::arch::kernel::interrupts::hardware::{interrupt_nested_disable, interrupt_nested_enable};

#[inline]
pub fn save_interrupt<F, R>(f: F) -> R
where
	F: FnOnce() -> R,
{
	let interrupt = interrupt_nested_disable();
	let output = f();

	interrupt_nested_enable(interrupt);

	output
}
