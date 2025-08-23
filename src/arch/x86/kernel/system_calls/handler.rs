// src/arch/x86/kernel/system_calls/handler.rs
use crate::syscall::SYSHANDLER_TABLE;
use core::arch::naked_asm;

#[naked]
pub(crate) unsafe extern "C" fn handle_system_call() {
    naked_asm!(
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "swapgs",
        "mov rcx, rsp",
        "rdgsbase rsp",
        "push rcx",
        "mov rcx, r10",
        "sti",
        "call [{sys_handler}+8*rax]",
        "cli",
        "pop rcx",
        "mov rsp, rcx",
        "swapgs",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "sysretq",
        sys_handler = sym SYSHANDLER_TABLE,
        options(noreturn));
}