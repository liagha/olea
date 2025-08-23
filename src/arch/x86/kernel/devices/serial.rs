use {
    crate::{
        sync::spinlock::SpinlockIrqSave,
    },
    core::fmt,
    x86::io::*,
};

pub(crate) struct SerialPort {
    base: u16,
}

impl SerialPort {
    const fn new(base: u16) -> Self {
        Self { base }
    }

    pub fn write_bytes(&mut self, buffer: &[u8]) {
        unsafe {
            for &b in buffer {
                outb(self.base, b);
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            for &b in s.as_bytes() {
                outb(self.base, b);
            }
        }

        Ok(())
    }
}

pub(crate) static COM1: SpinlockIrqSave<SerialPort> = SpinlockIrqSave::new(SerialPort::new(0x3F8));