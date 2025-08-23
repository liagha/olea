use {
    crate::{
        consts::USER_ENTRY,
    },
    core::arch::{naked_asm},
};

#[naked]
unsafe extern "C" fn jump_to_user_land(ds: usize, stack: usize, cs: usize, entry: usize) -> ! {
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

pub(crate) unsafe fn to_user_mode(func: usize) -> ! {
    jump_to_user_land(
        0x23,
        USER_ENTRY.as_usize() + 0x400000usize,
        0x2b,
        USER_ENTRY.as_usize() | func,
    )
}