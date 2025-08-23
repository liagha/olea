use crate::logging::*;
use crate::scheduler::*;
use crate::arch::x86::kernel::interrupts::exceptions::ExceptionStackFrame;
use crate::arch::x86::kernel::interrupts::hardware::{send_eoi_to_master, send_eoi_to_slave};

pub extern "x86-interrupt" fn unhandled_irq1(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    send_eoi_to_master();
}

pub extern "x86-interrupt" fn unhandled_irq2(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    send_eoi_to_slave();
    send_eoi_to_master();
}

pub extern "x86-interrupt" fn timer_handler(stack_frame: ExceptionStackFrame) {
    debug!(
		"task {} receive timer interrupt!\n{:#?}.",
		get_current_taskid(),
		stack_frame
	);

    send_eoi_to_master();
    schedule();
}