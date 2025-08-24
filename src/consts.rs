#![allow(dead_code)]

use {
    crate::{
        arch::memory::VirtAddr,
    }
};

/// Define the size of the kernel stack
pub const STACK_SIZE: usize = 0x3000;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub const INTERRUPT_STACK_SIZE: usize = 0x3000;

/// Size of a cache line
pub const CACHE_LINE: usize = 64;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 32;

pub const TIMER_FREQ: u32 = 100;

pub const USER_ENTRY: VirtAddr = VirtAddr(0x20000000000u64);

pub const HEAP_SIZE: usize = 8 * 1024 * 1024;
