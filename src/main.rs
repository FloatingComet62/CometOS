#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function


// Panic handler
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


// Main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
