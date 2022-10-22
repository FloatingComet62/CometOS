#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cometos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use cometos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[test_case]
fn test_println() {
    println!("test_println output");
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cometos::test_panic_handler(info)
}
