#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function
#![feature(custom_test_frameworks)]
#![test_runner(cometos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use cometos::println;

// Main
#[no_mangle] // don't change the function name, keep it _start
pub extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    cometos::init(); // IDT init
    
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    loop {}
}

// Panic handler
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cometos::test_panic_handler(info)
}

