#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function
#![feature(custom_test_frameworks)] // the rust test runner is included in std
#![test_runner(cometos::test_runner)] // defining the test runner
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{
    BootInfo,
    entry_point
};
#[allow(unused_imports)]
use cometos::{task::keyboard::ScancodeStream, print, println};

// Init and setup
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // INIT
    use cometos::memory::memory::{BootInfoFrameAllocator, init};
    use x86_64::VirtAddr;
    use cometos::memory::allocator;

    cometos::init(); // Initialize IDT and GDT
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = init(physical_memory_offset);
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    let _scancodestream = ScancodeStream::new();

    #[cfg(test)]
    test_main();

    main();

    cometos::hlt_loop();
}

fn main() {
    print!("\n> ");
}

// Panic handler
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    cometos::hlt_loop();
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cometos::test_panic_handler(info)
}
