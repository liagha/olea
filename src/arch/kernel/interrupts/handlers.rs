use {
    crate::{
        scheduler::*,
        arch::{
            kernel::{
                interrupts::{
                    end_of_interrupt, MASTER, SLAVE,
                    exceptions::ExceptionStackFrame,
                },
            },
        },
    },
};

pub extern "x86-interrupt" fn unhandled_irq1(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    end_of_interrupt(MASTER);
}

pub extern "x86-interrupt" fn unhandled_irq2(stack_frame: ExceptionStackFrame, irq: u64) {
    info!("task {} receive a unhandled IRQ: {} {:#?}.", get_current_taskid(), irq, stack_frame);
    end_of_interrupt(SLAVE);
    end_of_interrupt(MASTER);
}

pub extern "x86-interrupt" fn timer_handler(stack_frame: ExceptionStackFrame) {
    debug!(
		"task {} receive timer interrupt!\n{:#?}.",
		get_current_taskid(),
		stack_frame
	);

    end_of_interrupt(MASTER);
    schedule();
}