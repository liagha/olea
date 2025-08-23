// src/arch/x86/kernel/devices/serial.rs
use crate::sync::spinlock::SpinlockIrqSave;
use core::fmt;
use x86::io::*;

pub(crate) struct SerialPort {
    base_addr: u16,
}

impl SerialPort {
    const fn new(base_addr: u16) -> Self {
        Self { base_addr }
    }

    pub fn write_bytes(&mut self, buf: &[u8]) {
        unsafe {
            for &b in buf {
                outb(self.base_addr, b);
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            for &b in s.as_bytes() {
                outb(self.base_addr, b);
            }
        }

        Ok(())
    }
}

pub(crate) static COM1: SpinlockIrqSave<SerialPort> = SpinlockIrqSave::new(SerialPort::new(0x3F8));