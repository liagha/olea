use {
    crate::{
        consts::USER_ENTRY,
    },
    core::arch::{naked_asm},
};

/// Low-level function to transition from kernel mode to user mode
/// Uses iretq instruction which pops values from stack to restore user context
///
/// Stack layout for iretq (pushed in this order):
/// - Code segment selector (cs)
/// - Instruction pointer (entry)
/// - Flags register
/// - Stack segment selector (ss, derived from ds)
/// - Stack pointer (stack)
#[naked]
unsafe extern "C" fn jump_to_user_land(ds: usize, stack: usize, cs: usize, entry: usize) -> ! {
    naked_asm!(
        // Switch to user GS before transitioning
        "swapgs",

        // Set up stack for iretq instruction
        // iretq pops these in reverse order to restore user context
        "push rdi",    // ds (data segment) - becomes SS (stack segment)
        "push rsi",    // stack pointer for user mode
        "pushf",       // current flags register
        "push rdx",    // cs (code segment)
        "push rcx",    // entry point (instruction pointer)

        // Interrupt return - pops all values above and switches to user mode
        // This is the only way to drop from kernel privilege to user privilege
        "iretq",

        options(noreturn)  // Never returns
    )
}

/// High-level wrapper to transition to user mode with predefined segments
/// Sets up proper user-space segment selectors and memory layout
pub unsafe fn to_user_mode(func: usize) -> ! {
    jump_to_user_land(
        0x23,  // User data segment selector (GDT entry 4, DPL=3)
        USER_ENTRY.as_usize() + 0x400000usize,  // User stack pointer
        0x2b,  // User code segment selector (GDT entry 5, DPL=3)
        USER_ENTRY.as_usize() | func,           // User entry point
    )
}