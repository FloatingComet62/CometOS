use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};
use linked_list::LinkedListAllocator;
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB



pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }

    Ok(())
}

pub struct Dummy;
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
// a wrapper around spin::Mutex to permit trait implentaions on:
// "unsafe impl GlobalAlloc for spin::Mutex<BumpAllocator>"
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}
impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

//  Align the given `address` upwards to alignment `align`
///  align MUST be a power of 2
fn align_up(address: usize, align: usize) -> usize {
    // how the expression below works:
    // * since align is a power of 2, it's binary representation has only a single bit set
    //   (eg. `0b00010000`). This mean that `align - 1` has all the lower bits set
    //   (eg. `0b00001111`).
    // * by creating the bitwise NOT through `!` operator, we get a number that has all the bits
    //   set except for the bits lower than `align`
    //   (eg. `0b11110000`)
    // * by performing a bitwise AND on an address and !(align - 1), we align the adddress
    //   downwards. This works by clearing all the bits that are lower than align.
    // * Since we want to align upwards instead of downwards, we increase `address` by `align - 1`
    //   before performing the bitwise AND. This way, we already aligned addresses remain the same
    //   while non-aligned address are rounded to the next alignment boundary.
    (address + align - 1) & !(align - 1)

    /*
     * The above code is much faster, but these mean the exact same
     *  let remainder = address % align;
     *  if remainder == 0 {
     *      address // address already aligned
     *  } else {
     *      address - remainder + align
     *  }
     */
}
