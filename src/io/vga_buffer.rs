use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

pub mod color;
pub mod writer;

// constants are initialized at compile time, this allows us to initialize constants at runtime
lazy_static! {
    pub static ref WRITER: Mutex<writer::Writer> = Mutex::new(writer::Writer {
        column_position: 0,
        color_code: color::ColorCode::new(color::Color::Yellow, color::Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut writer::Buffer) }, // 0xb8000 is a mutable raw pointer
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
}

