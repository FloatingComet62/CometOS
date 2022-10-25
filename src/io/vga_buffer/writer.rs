use volatile::Volatile;
use core::fmt;
// Character
// Bits       Value
// 0-7        ASCII code point
// 8-11       Foreground color
// 12-14      Background color
// 15         Blink
//
// Since the field ordering in default structs is undefined in Rust, we need the repr(C) attribute.
// It guarantees that the struct's fields are laid out exactly like in a C struct and thus
// guarantees the coreect field ordering. For the Buffer struct, we use repr(transparent) agaain to
// ensure that it has the same memory layout as its single field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: super::color::ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [
        [
            Volatile<ScreenChar>; // Cargo.toml | Line 16 
            BUFFER_WIDTH
        ];
        BUFFER_HEIGHT
    ],
}

// The writer will always write to the last line and shift lines up when a line is full(or on \n).
// The column_position field keeps tract of the current position in the last row. The current
// foreground and background colors are specified by color_code and a reference to the VGA buffer
// is stored in buffer. Note that we need an explicit lifetime here to tell the compiler hwo long
// the reference is valid. The 'static lifetime specifies that the reference is valid for the whole
// program run time(which is true for the VGA text buffer).
pub struct Writer {
    pub column_position: usize,
    pub color_code: super::color::ColorCode,
    pub buffer: &'static mut Buffer,
}
impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // valid ASCII character
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // invalid character
                _ => self.write_byte(0xfe)
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
