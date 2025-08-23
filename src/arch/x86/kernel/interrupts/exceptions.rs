use {
    crate::{
        logging::*,
        scheduler::*,
        arch::x86::kernel::interrupts::hardware::{
            MASTER,
            end_of_interrupt,
        },
    },
    core::fmt,
};

#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl fmt::Debug for ExceptionStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

pub extern "x86-interrupt" fn handle_divide_by_zero_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Divide By Zero Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn debug_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Debug Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn nmi_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Non Maskable Interrupt Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn int3_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Int 3 Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn int0_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a INT0 Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn out_of_bound_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Out of Bounds Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn invalid_opcode_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Invalid Opcode Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn no_coprocessor_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Coprocessor Not Available Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn double_fault_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a Double Fault Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    abort();
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Coprocessor Segment Overrun Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn bad_tss_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a Bad TSS Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn segment_not_present_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a Segment Not Present Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn stack_fault_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a Stack Fault Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn general_protection_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a General Protection Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn reserved_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Reserved Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn floating_point_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Floating Point Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn alignment_check_exception(stack_frame: ExceptionStackFrame, error_code: u64) {
    info!("task {} receive a Alignment Check Exception: {:#?} error code: 0x{:x}.", get_current_taskid(), stack_frame, error_code);
    end_of_interrupt(MASTER);
    abort();
}

pub extern "x86-interrupt" fn machine_check_exception(stack_frame: ExceptionStackFrame) {
    info!("task {} receive a Machine Check Exception: {:#?}.", get_current_taskid(), stack_frame);
    end_of_interrupt(MASTER);
    abort();
}