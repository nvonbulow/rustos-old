use super::*;
use core::ptr::Unique;
use core::fmt;
use spin::Mutex;
use x86::shared::io::outb;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    row_position: 0,
    column_position: 0,
    color: ColorCode::new(Color::White, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) },
});

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::text::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn clear_buffer() {
    let mut writer = WRITER.lock();
    writer.clear_buffer();
    writer.row_position = 0;
    writer.column_position = 0;
    update_cursor(0, 0);
}

fn update_cursor(row: usize, column: usize) {
    let position: u16 = (row * 80 + column) as u16;

    unsafe {
        //If cursor breaks, then it's because we have the wrong ports
        outb(0x3D4, 0x0F);
        outb(0x3D5, position as u8);
        outb(0x3D4, 0x0E);
        outb(0x3D5, (position >> 8) as u8);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Character {
    ascii: u8,
    color_code: ColorCode
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


struct Buffer {
    chars: [[Character; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color: ColorCode,
    buffer: Unique<Buffer>,
}

#[allow(dead_code)]
impl Writer {
    pub fn write_byte(&mut self, ascii: u8) {
        match ascii {
            b'\n' => self.new_line(),
            ascii => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row_position;
                let col = self.column_position;
                let color= self.color;
                self.buffer().chars[row][col] = Character {
                    ascii: ascii,
                    color_code: color
                };
                self.column_position += 1;
            }
        }
        update_cursor(self.row_position, self.column_position);
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe {
            self.buffer.get_mut()
        }
    }

    pub fn new_line(&mut self) {
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let buffer = self.buffer();
                    let character = buffer.chars[row][col];
                    buffer.chars[row - 1][col] = character;
                }
            }
            self.clear_row(BUFFER_HEIGHT-1);
            self.column_position = 0;
            self.row_position = BUFFER_HEIGHT - 2;
        }
        self.row_position += 1;
        self.column_position = 0;
        update_cursor(self.row_position, self.column_position);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Character {
            ascii: b' ',
            color_code: self.color
        };
        for column in 0..BUFFER_WIDTH {
            self.buffer().chars[row][column] = blank;
        }
    }

    pub fn clear_buffer(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        update_cursor(self.row_position, self.column_position);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
