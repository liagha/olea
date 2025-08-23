use core::arch::asm;

// src/arch/x86/kernel/system_calls/interface.rs
#[macro_export]
macro_rules! system_call {
    ($arg0:expr) => {
        arch::x86::kernel::invoke_system_call_0_args($arg0 as u64)
    };

    ($arg0:expr, $arg1:expr) => {
        arch::x86::kernel::invoke_system_call_1_args($arg0 as u64, $arg1 as u64)
    };

    ($arg0:expr, $arg1:expr, $arg2:expr) => {
        arch::x86::kernel::invoke_system_call_2_args($arg0 as u64, $arg1 as u64, $arg2 as u64)
    };

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
        arch::x86::kernel::invoke_system_call_3_args($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64)
    };

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        arch::x86::kernel::invoke_system_call_4_args(
            $arg0 as u64,
            $arg1 as u64,
            $arg2 as u64,
            $arg3 as u64,
            $arg4 as u64,
        )
    };

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
        arch::x86::kernel::invoke_system_call_5_args(
            $arg0 as u64,
            $arg1 as u64,
            $arg2 as u64,
            $arg3 as u64,
            $arg4 as u64,
            $arg5 as u64,
        )
    };

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {
        arch::x86::kernel::invoke_system_call_6_args(
            $arg0 as u64,
            $arg1 as u64,
            $arg2 as u64,
            $arg3 as u64,
            $arg4 as u64,
            $arg5 as u64,
            $arg6 as u64,
        )
    };

    ($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr, $arg7:expr) => {
        arch::x86::kernel::invoke_system_call_7_args(
            $arg0 as u64,
            $arg1 as u64,
            $arg2 as u64,
            $arg3 as u64,
            $arg4 as u64,
            $arg5 as u64,
            $arg6 as u64,
            $arg7 as u64,
        )
    };
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_0_args(arg0: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_1_args(arg0: u64, arg1: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_2_args(arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_3_args(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_4_args(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_5_args(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        in("r8") arg5,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn invoke_system_call_6_args(
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("syscall",
        inlateout("rax") arg0 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        in("r8") arg5,
        in("r9") arg6,
        lateout("rcx") _,
        lateout("r11") _,
        options(preserves_flags, nostack)
        );
    }
    ret
}