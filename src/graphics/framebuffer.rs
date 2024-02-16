use volatile::Volatile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8,
    pub row: usize,
    pub column: usize,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [
        [
            Volatile<Pixel>; // Cargo.toml | Line 16 
            BUFFER_WIDTH
        ];
        BUFFER_HEIGHT
    ],
}

pub struct FrameBuffer {
    pub buffer: &'static mut Buffer,
}
impl FrameBuffer {
    pub fn new() -> Self {
        return Self {
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) } // 0xb8000 is a mutable raw pointer
        };
    }
    pub fn write_pixel(&mut self, pixel: Pixel) {
        self.buffer.chars[pixel.row][pixel.column].write(pixel);
    }
}
