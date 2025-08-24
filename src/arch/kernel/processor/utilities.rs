use {
    crate::arch::asm,
};

#[inline(always)]
pub fn memory_barrier() {
    unsafe {
        asm!("mfence", options(preserves_flags, nostack));
    }
}

#[inline(always)]
pub fn most_significant_bit(value: usize) -> Option<usize> {
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
pub fn least_significant_bit(value: usize) -> Option<usize> {
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
pub fn cpu_halt() {
    unsafe {
        asm!("hlt", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn pause() {
    unsafe {
        asm!("pause", options(nomem, nostack));
    }
}