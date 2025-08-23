// src/arch/x86/kernel/system_calls/user_transition.rs
use crate::consts::USER_ENTRY;
use core::arch::{naked_asm};

#[naked]
unsafe extern "C" fn __jump_to_user_land(ds: usize, stack: usize, cs: usize, entry: usize) -> ! {
    naked_asm!(
        "swapgs",
        "push rdi",
        "push rsi",
        "pushf",
        "push rdx",
        "push rcx",
        "iretq",
        options(noreturn)
    )
}

pub(crate) unsafe fn transition_to_user_mode(func: usize) -> ! {
    __jump_to_user_land(
        0x23,
        USER_ENTRY.as_usize() + 0x400000usize,
        0x2b,
        USER_ENTRY.as_usize() | func,
    )
}