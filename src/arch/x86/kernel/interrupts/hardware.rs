// src/arch/x86/kernel/interrupts/hardware.rs
use core::arch::asm;
use x86::io::*;
use crate::arch::x86::kernel::descriptors::interrupts::INTERRUPT_HANDLER;

pub fn irq_enable() {
    unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
}

pub fn irq_disable() {
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
}

pub fn is_irq_enabled() -> bool {
    let eflags: usize;

    unsafe { asm!("pushf; pop {}", lateout(reg) eflags, options(nomem, nostack, preserves_flags)) };
    if (eflags & (1usize << 9)) != 0 {
        return true;
    }

    false
}

pub fn irq_nested_disable() -> bool {
    let was_enabled = is_irq_enabled();
    irq_disable();
    was_enabled
}

pub fn irq_nested_enable(was_enabled: bool) {
    if was_enabled {
        irq_enable();
    }
}

#[inline(always)]
pub(crate) fn send_eoi_to_slave() {
    unsafe {
        outb(0xA0, 0x20);
    }
}

#[inline(always)]
pub(crate) fn send_eoi_to_master() {
    unsafe {
        outb(0x20, 0x20);
    }
}

unsafe fn irq_remap() {
    outb(0x20, 0x11);
    outb(0xA0, 0x11);
    outb(0x21, 0x20);
    outb(0xA1, 0x28);
    outb(0x21, 0x04);
    outb(0xA1, 0x02);
    outb(0x21, 0x01);
    outb(0xA1, 0x01);
    outb(0x21, 0x00);
    outb(0xA1, 0x00);
}

pub(crate) fn init() {
    unsafe {
        irq_remap();

        INTERRUPT_HANDLER.lock().load_idt();
    }
}