// src/arch/x86/kernel/processor/utilities.rs
use core::arch::asm;

#[inline(always)]
pub(crate) fn memory_barrier() {
    unsafe {
        asm!("mfence", options(preserves_flags, nostack));
    }
}

#[inline(always)]
pub(crate) fn most_significant_bit(value: usize) -> Option<usize> {
    if value > 0 {
        let ret: usize;

        unsafe {
            asm!("bsr {0}, {1}",
            out(reg) ret,
            in(reg) value,
            options(nomem, nostack)
            );
        }
        Some(ret)
    } else {
        None
    }
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn lsb(value: usize) -> Option<usize> {
    if value > 0 {
        let ret: usize;
        unsafe {
            asm!("bsf {0}, {1}",
            out(reg) ret,
            in(reg) value,
            options(nomem, nostack)
            );
        }
        Some(ret)
    } else {
        None
    }
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn cpu_halt() {
    unsafe {
        asm!("hlt", options(nomem, nostack));
    }
}

#[inline(always)]
pub(crate) fn pause() {
    unsafe {
        asm!("pause", options(nomem, nostack));
    }
}