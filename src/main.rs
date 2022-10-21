#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function

mod vga_buffer;

// Panic handler
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Main
#[no_mangle] // don't change the function name, keep it _start
pub extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    loop {}
}
