use crate::arch::x86::kernel::interrupts::hardware::{irq_nested_disable, irq_nested_enable};

#[inline]
pub fn irqsave<F, R>(f: F) -> R
where
	F: FnOnce() -> R,
{
	let irq = irq_nested_disable();
	let ret = f();
	irq_nested_enable(irq);
	ret
}
