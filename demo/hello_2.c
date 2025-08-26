#include <stddef.h>
#include <stdint.h>

// System call numbers matching your kernel
#define SYS_WRITE 1
#define SYS_WRITEV 20
#define SYS_EXIT 60

// File descriptors
#define STDOUT_FD 1
#define STDERR_FD 2

// Structure for writev (matching your BufferSegment)
struct iovec {
    const void *base;
    size_t length;
};

// Inline assembly wrapper for system calls
static inline long syscall1(long number, long arg1) {
    long result;
    __asm__ volatile (
        "syscall"
        : "=a" (result)
        : "a" (number), "D" (arg1)
        : "rcx", "r11", "memory"
    );
    return result;
}

static inline long syscall3(long number, long arg1, long arg2, long arg3) {
    long result;
    __asm__ volatile (
        "syscall"
        : "=a" (result)
        : "a" (number), "D" (arg1), "S" (arg2), "d" (arg3)
        : "rcx", "r11", "memory"
    );
    return result;
}

// System call wrappers
static long write(int fd, const void *buffer, size_t count) {
    return syscall3(SYS_WRITE, fd, (long)buffer, count);
}

static long writev(int fd, const struct iovec *iov, int iovcnt) {
    return syscall3(SYS_WRITEV, fd, (long)iov, iovcnt);
}

static void exit(int status) {
    syscall1(SYS_EXIT, status);
    __builtin_unreachable();
}

// String length function
static size_t strlen(const char *str) {
    size_t len = 0;
    while (str[len]) len++;
    return len;
}

// Entry point function (must be named _start for freestanding programs)
void _start(void) {
    // Test 1: Simple stack-allocated string (should work)
    char simple[] = {'H', 'e', 'l', 'l', 'o', '\n'};
    long result1 = write(STDOUT_FD, simple, 6);

    // Test 2: Try a string literal (might fail due to memory mapping)
    long result2 = write(STDOUT_FD, "World\n", 6);

    // Test 3: Print the return values to see what's happening
    // Convert result to single digit (assuming small numbers)
    if (result1 >= 0) {
        char success1[] = {'1', 'O', 'K', '\n'};
        write(STDOUT_FD, success1, 4);
    } else {
        char fail1[] = {'1', 'E', 'R', 'R', '\n'};
        write(STDOUT_FD, fail1, 5);
    }

    if (result2 >= 0) {
        char success2[] = {'2', 'O', 'K', '\n'};
        write(STDOUT_FD, success2, 4);
    } else {
        char fail2[] = {'2', 'E', 'R', 'R', '\n'};
        write(STDOUT_FD, fail2, 5);
    }

    // Exit cleanly
    exit(0);
}