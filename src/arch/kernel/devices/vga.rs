use {
    crate::{
        format,
        sync::lock::WaitLock,
        arch::x86::outb,
    },
};

const CRT_CONTROLLER_ADDRESS_PORT: u16 = 0x3D4;
const CRT_CONTROLLER_DATA_PORT: u16 = 0x3D5;
const CURSOR_START_REGISTER: u8 = 0x0A;
const CURSOR_DISABLE: u8 = 0x20;

const ATTRIBUTE_BLACK: u8 = 0x00;
const ATTRIBUTE_LIGHTGREY: u8 = 0x07;
const COLS: usize = 80;
const ROWS: usize = 25;
const VGA_BUFFER_ADDRESS: u64 = 0xB8000;

pub static VGA_SCREEN: WaitLock<VgaScreen> = WaitLock::new(VgaScreen::new());

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct VgaCharacter {
    character: u8,
    attribute: u8,
}

impl VgaCharacter {
    const fn new(character: u8, attribute: u8) -> Self {
        Self {
            character,
            attribute,
        }
    }
}

pub struct VgaScreen {
    buffer: *mut [[VgaCharacter; COLS]; ROWS],
    current_col: usize,
    current_row: usize,
    is_initialized: bool,
}

impl VgaScreen {
    const fn new() -> Self {
        Self {
            buffer: VGA_BUFFER_ADDRESS as *mut _,
            current_col: 0,
            current_row: 0,
            is_initialized: false,
        }
    }

    fn init(&mut self) {
        unsafe {
            outb(CRT_CONTROLLER_ADDRESS_PORT, CURSOR_START_REGISTER);
            outb(CRT_CONTROLLER_DATA_PORT, CURSOR_DISABLE);
        }

        for r in 0..ROWS {
            self.clear_row(r);
        }

        self.is_initialized = true;
    }

    #[inline]
    fn clear_row(&mut self, row: usize) {
        for c in 0..COLS {
            unsafe {
                (*self.buffer)[row][c] = VgaCharacter::new(0, ATTRIBUTE_BLACK);
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if !self.is_initialized {
            return;
        }

        if byte == b'\n' || self.current_col == COLS {
            self.current_col = 0;
            self.current_row += 1;
        }

        if self.current_row == ROWS {
            for r in 1..ROWS {
                for c in 0..COLS {
                    unsafe {
                        (*self.buffer)[r - 1][c] = (*self.buffer)[r][c];
                    }
                }
            }

            self.clear_row(ROWS - 1);
            self.current_row = ROWS - 1;
        }

        if byte != b'\n' {
            unsafe {
                (*self.buffer)[self.current_row][self.current_col] =
                    VgaCharacter::new(byte, ATTRIBUTE_LIGHTGREY);
            }
            self.current_col += 1;
        }
    }

    pub fn write_bytes(&mut self, buf: &[u8]) {
        for &b in buf {
            self.write_byte(b);
        }
    }
}

unsafe impl Send for VgaScreen {}

impl format::Write for VgaScreen {
    fn write_str(&mut self, s: &str) -> format::Result {
        for &b in s.as_bytes() {
            self.write_byte(b);
        }

        Ok(())
    }
}

pub fn init() {
    VGA_SCREEN.lock().init();
}