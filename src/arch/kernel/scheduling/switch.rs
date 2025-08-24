use {
    crate::arch::memory::VirtAddr,
    core::arch::naked_asm,
};
use crate::arch::kernel::descriptors::global::set_current_kernel_stack;

#[cfg(target_arch = "x86_64")]
macro_rules! save_context {
    () => {
        concat!(
            r#"
            pushfq
            push rax
            push rcx
            push rdx
            push rbx
            sub  rsp, 8
            push rbp
            push rsi
            push rdi
            push r8
            push r9
            push r10
            push r11
            push r12
            push r13
            push r14
            push r15
            "#,
        )
    };
}

#[cfg(target_arch = "x86_64")]
macro_rules! restore_context {
    () => {
        concat!(
            r#"
            pop r15
            pop r14
            pop r13
            pop r12
            pop r11
            pop r10
            pop r9
            pop r8
            pop rdi
            pop rsi
            pop rbp
            add rsp, 8
            pop rbx
            pop rdx
            pop rcx
            pop rax
            popfq
            ret
            "#
        )
    };
}

#[cfg(target_arch = "x86_64")]
#[naked]
pub unsafe extern "C" fn perform_context_switch(_old_stack: *mut VirtAddr, _new_stack: VirtAddr) {
    naked_asm!(
        save_context!(),
        "rdfsbase rax",
        "rdgsbase rdx",
        "push rax",
        "push rdx",
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "mov rax, cr0",
        "or rax, 8",
        "mov cr0, rax",
        "call {set_stack}",
        "pop r15",
        "wrgsbase r15",
        "pop r15",
        "wrfsbase r15",
        restore_context!(),
        set_stack = sym set_current_kernel_stack,
        options(noreturn)
    );
}

#[cfg(target_arch = "x86")]
#[naked]
pub unsafe extern "C" fn perform_context_switch(_old_stack: *mut usize, _new_stack: usize) {
    naked_asm!(
        "pushfd",
        "pushad",
        "mov edi, [esp+10*4]",
        "mov [edi], esp",
        "mov esp, [esp+11*4]",
        "popad",
        "popfd",
        "ret",
        options(noreturn)
    );
}