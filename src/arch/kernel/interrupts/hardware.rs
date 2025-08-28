use {
    crate::{
        arch::{
            asm,
            kernel::{
                descriptors::interrupts::INTERRUPT_HANDLER,
            },
            x86::outb,
        },
    },
};

pub const MASTER: u16 = 0x20;
pub const SLAVE: u16 = 0xA0;

pub fn interrupt_enable() {
    unsafe { asm!("sti", options(nomem, nostack, preserves_flags)) };
}

pub fn interrupt_disable() {
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };
}

pub fn is_interrupt_enabled() -> bool {
    let eflags: usize;

    unsafe { asm!("pushf; pop {}", lateout(reg) eflags, options(nomem, nostack, preserves_flags)) };
    if (eflags & (1usize << 9)) != 0 {
        return true;
    }

    false
}

pub fn interrupt_nested_disable() -> bool {
    let was_enabled = is_interrupt_enabled();
    interrupt_disable();
    was_enabled
}

pub fn interrupt_nested_enable(was_enabled: bool) {
    if was_enabled {
        interrupt_enable();
    }
}

#[inline(always)]
pub fn end_of_interrupt(port: u16) {
    unsafe {
        outb(port, 0x20);
    }
}

unsafe fn interrupt_remap() {
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

pub fn initialize() {
    unsafe {
        interrupt_remap();

        INTERRUPT_HANDLER.lock().load_interrupts();
    }
}