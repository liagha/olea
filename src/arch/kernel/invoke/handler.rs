use {
    crate::{
        arch::naked_asm,
        invoke::INVOKE_TABLE,
    },
};

#[naked]
pub unsafe extern "C" fn invoke_handler() {
    naked_asm!(
        // === SAVE USER CONTEXT ===
        // Save all caller-saved registers that might be clobbered
        // Note: rax contains invoke number, so we don't save it yet
        "push rcx",     // User return address (set by invoke instruction)
        "push rdx",     // 3rd invoke argument
        "push rsi",     // 2nd invoke argument
        "push rdi",     // 1st invoke argument
        "push r8",      // 5th invoke argument
        "push r9",      // 6th invoke argument
        "push r10",     // 4th invoke argument (will be moved to rcx later)
        "push r11",     // User flags register (set by invoke instruction)

        // === SWITCH TO KERNEL CONTEXT ===
        // Switch GS segment register from user to kernel
        // This allows access to kernel per-CPU data structures
        "swapgs",

        // === STACK SWITCH ===
        // Switch from user stack to kernel stack for safety/isolation
        "mov rcx, rsp",        // Save current user stack pointer
        "rdgsbase rsp",        // Load kernel stack pointer from GS base
        "push rcx",            // Save user stack pointer on kernel stack

        // === PREPARE SYSCALL ARGUMENTS ===
        // Adjust register layout to match expected calling convention
        // r10 contains 4th argument but C calling convention expects it in rcx
        "mov rcx, r10",        // Move 4th argument from r10 to rcx

        // === ENABLE INTERRUPTS AND DISPATCH ===
        // Re-enable interrupts (disabled during privilege switch)
        "sti",

        // Call the appropriate system invoke handler
        // rax contains invoke number, used as index into handler table
        // Each entry is 8 bytes (pointer size), so multiply by 8
        "call [{sys_handler}+8*rax]",

        // === RESTORE KERNEL CONTEXT ===
        // Disable interrupts before switching back to userspace
        "cli",

        // === STACK SWITCH BACK ===
        "pop rcx",             // Restore user stack pointer
        "mov rsp, rcx",        // Switch back to user stack

        // === SWITCH BACK TO USER CONTEXT ===
        // Switch GS back to user mode
        "swapgs",

        // === RESTORE USER REGISTERS ===
        // Restore all saved registers in reverse order
        "pop r11",             // User flags (will be restored by sysretq)
        "pop r10",             // 4th argument
        "pop r9",              // 6th argument
        "pop r8",              // 5th argument
        "pop rdi",             // 1st argument
        "pop rsi",             // 2nd argument
        "pop rdx",             // 3rd argument
        "pop rcx",             // User return address (will be restored by sysretq)

        // === RETURN TO USERSPACE ===
        // Special instruction to return from invoke to userspace
        // Restores user CS/SS, jumps to address in rcx, restores flags from r11
        "sysretq",

        // Tell assembler that SYSHANDLER_TABLE symbol should be used
        sys_handler = sym INVOKE_TABLE,
        options(noreturn)  // This function never returns normally
    );
}