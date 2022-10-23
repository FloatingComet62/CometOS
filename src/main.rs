#![no_std] // we don't want to include C standard library
#![no_main] // since we don't include std, there is no main calling function
#![feature(custom_test_frameworks)] // the rust test runner is included in std
#![test_runner(cometos::test_runner)] // defining the test runner
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};
use cometos::println;


// Main
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use cometos::memory::{BootInfoFrameAllocator, init, create_example_mapping};
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello world{}", "!");
    cometos::init(); // Initialize IDT and GDT
    
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = init(physical_memory_offset);
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // 0x_f021_f077_f065_f04e -> New! (string)
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

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
