#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function
#![feature(custom_test_frameworks)] // the rust test runner is included in std
#![test_runner(cometos::test_runner)] // defining the test runner
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use cometos::println;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};


// Main
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use cometos::memory::memory::{BootInfoFrameAllocator, init};
    use x86_64::VirtAddr;
    use cometos::memory::allocator;

    println!("Hello world{}", "!");
    cometos::init(); // Initialize IDT and GDT
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = init(physical_memory_offset);
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    #[cfg(test)]
    test_main();

    println!("[ok]");

    cometos::hlt_loop();
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
