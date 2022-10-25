use super::{
    align_up,
    Locked
};
use alloc::alloc::{
    GlobalAlloc,
    Layout
};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}
impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

// We have to use the mutex because in alloc function, the first argument is &self and not &mut
// self, so we can't edit the data...
// There is a way to get a &mut self reference from a &self reference: We can yse synchronized
// interior mutability by wrapping the allocator in spin::Mutex spinlock. This type provides a lock
// method that performs mutual exclusion and this safely turn &self reference to a &mut self
// reference.
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            return ptr::null_mut() // out of memory
        }

        bump.next = alloc_end;
        bump.allocations += 1;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

