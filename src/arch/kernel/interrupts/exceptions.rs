use {
    crate::{
        format,
        scheduler::*,
        arch::{
            kernel::{
                interrupts::{
                    end_of_interrupt, MASTER,
                },
            },
        },
    },
};

#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl format::Debug for ExceptionStackFrame {
    fn fmt(&self, f: &mut format::Formatter) -> format::Result {
        struct Hex(u64);
        impl format::Debug for Hex {
            fn fmt(&self, f: &mut format::Formatter) -> format::Result {
                write!(f, "{:#x}", self.0)
            }
        }

        let mut s = f.debug_struct("ExceptionStackFrame");
        s.field("instruction_pointer", &Hex(self.instruction_pointer));
        s.field("code_segment", &Hex(self.code_segment));
        s.field("cpu_flags", &Hex(self.cpu_flags));
        s.field("stack_pointer", &Hex(self.stack_pointer));
        s.field("stack_segment", &Hex(self.stack_segment));
        s.finish()
    }
}

pub extern "x86-interrupt" fn divide_by_zero(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Divide By Zero` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn debug(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Debug` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn non_maskable(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Non Maskable Interrupt` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn int_three(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Int 3` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn int_zero(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `INT0` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn out_of_bound(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Out of Bounds` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Invalid Opcode` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn no_coprocessor(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Coprocessor Not Available` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn double_fault(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `Double Fault` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    abort();
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Coprocessor Segment Overrun` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn bad_tss(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `Bad TSS` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn segment_not_present(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `Segment Not Present` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn stack_fault(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `Stack Fault` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn general_protection(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `General Protection` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn reserved(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Reserved` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn floating_point(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Floating Point` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn alignment_check(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task `{}` receive a `Alignment Check` exception: `{:#?}` error code: `0x{:x}`.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn machine_check(stack_frame: ExceptionStackFrame) {
    info!("task `{}` receive a `Machine Check` exception: `{:#?}`.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}