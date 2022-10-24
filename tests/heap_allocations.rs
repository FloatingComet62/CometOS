#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cometos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use cometos::memory::allocator;
    use cometos::memory::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    cometos::init();
    let physical_memmory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = memory::init(physical_memmory_offset);
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    test_main();
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cometos::test_panic_handler(info)
}

use alloc::boxed::Box;

#[test_case]
fn simple_allocation() {
    let heap_value1 = Box::new(41);
    let heap_value2 = Box::new(13);
    assert_eq!(*heap_value1, 41);
    assert_eq!(*heap_value2, 13);
}

use alloc::vec::Vec;

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n-1) * n / 2); // Σn = n(n-1)/2
}

use cometos::memory::allocator::HEAP_SIZE;

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
