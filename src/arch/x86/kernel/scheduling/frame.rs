// src/arch/x86/kernel/scheduling/frame.rs
use crate::arch::memory::VirtAddr;
use crate::consts::*;
use crate::logging::*;
use crate::scheduler::task::*;
use crate::scheduler::{do_exit, get_current_taskid};
use core::mem::size_of;
use core::ptr::write_bytes;

#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
struct State {
    gs: u64,
    fs: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rsp: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    rflags: u64,
    rip: u64,
}

#[cfg(target_arch = "x86")]
#[repr(C, packed)]
struct State {
    edi: u32,
    esi: u32,
    ebp: u32,
    esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    eflags: u32,
    eip: u32,
}

extern "C" fn leave_task() -> ! {
    debug!("finished task {}.", get_current_taskid());

    do_exit();
}

impl TaskFrame for Task {
    #[cfg(target_arch = "x86_64")]
    fn create_stack_frame(&mut self, func: extern "C" fn()) {
        unsafe {
            let mut stack: *mut u64 = (*self.stack).top().as_mut_ptr();

            write_bytes((*self.stack).bottom().as_mut_ptr::<u8>(), 0xCD, STACK_SIZE);

            *stack = 0xDEADBEEFu64;
            stack = (stack as usize - size_of::<u64>()) as *mut u64;

            *stack = (leave_task as *const ()) as u64;
            stack = (stack as usize - size_of::<State>()) as *mut u64;

            let state: *mut State = stack as *mut State;
            write_bytes(state, 0x00, 1);

            (*state).rsp = (stack as usize + size_of::<State>()) as u64;
            (*state).rbp = (*state).rsp + size_of::<u64>() as u64;
            (*state).gs = (*self.stack).top().as_u64();

            (*state).rip = (func as *const ()) as u64;
            (*state).rflags = 0x1202u64;

            self.last_stack_pointer = VirtAddr(stack as u64);
        }
    }

    #[cfg(target_arch = "x86")]
    fn prepare_initial_task_frame(&mut self, func: extern "C" fn()) {
        unsafe {
            let mut stack: *mut u32 = ((*self.stack).top()).as_mut_ptr();

            write_bytes((*self.stack).bottom().as_mut_ptr::<u8>(), 0xCD, STACK_SIZE);

            *stack = 0xDEADBEEFu32;
            stack = (stack as usize - size_of::<u32>()) as *mut u32;

            *stack = (leave_task as *const ()) as u32;
            stack = (stack as usize - size_of::<State>()) as *mut u32;

            let state: *mut State = stack as *mut State;
            write_bytes(state, 0x00, 1);

            (*state).esp = (stack as usize + size_of::<State>()) as u32;
            (*state).ebp = (*state).esp + size_of::<u32>() as u32;

            (*state).eip = (func as *const ()) as u32;
            (*state).eflags = 0x1002u32;

            self.last_stack_pointer = stack as usize;
        }
    }
}