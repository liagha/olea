void _start() {
    const char msg[] = "Hello\n";
    asm volatile(
        "mov $1, %%rax\n"      // sys_write
        "mov $1, %%rdi\n"      // fd=1
        "lea %0, %%rsi\n"      // buf
        "mov $6, %%rdx\n"      // len
        "syscall\n"
        "mov $60, %%rax\n"     // sys_exit
        "xor %%rdi, %%rdi\n"
        "syscall\n"
        : : "m"(msg) : "rax","rdi","rsi","rdx"
    );
}
