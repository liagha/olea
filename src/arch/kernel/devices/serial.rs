use {
    crate::{
        format,
        sync::lock::WaitLockIrqSave,
        arch::x86::outb,
    },
};

pub struct SerialPort {
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

impl format::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> format::Result {
        unsafe {
            for &b in s.as_bytes() {
                outb(self.base, b);
            }
        }

        Ok(())
    }
}

pub static PORT: WaitLockIrqSave<SerialPort> = WaitLockIrqSave::new(SerialPort::new(0x3F8));