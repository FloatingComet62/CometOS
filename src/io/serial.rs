// Index:
// Imports        6
// SERIAL1 static 17
// _print()       27

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

// A simple way to send data is to use the serial port, an old interface standard which is no
// longer found in modern computers. It is easy to program and QEMU can redirect the bytes to send
// over serial to the host's standard output or a file.
//
// The chip implementing a serial interface are called UARTs. The common UARTs today are all
// compatible with the 16550 UART.

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // port 0x3F8 is the standard port for  the first serial inferface
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    })
}
