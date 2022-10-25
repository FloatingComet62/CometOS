use core::{
    mem,
    ptr
};
use super::{
    Locked,
    align_up
};
use alloc::alloc::{
    GlobalAlloc,
    Layout
};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}
impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    } 

    fn start_address(&self) -> usize {
        self as *const Self as usize
    }

    fn end_address(&self) -> usize {
        self.start_address() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}
impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0)
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    // Initialize the allocator with the given heap bounds.
    unsafe fn add_free_region(&mut self, address: usize, size: usize) {
        assert_eq!(align_up(address, mem::align_of::<ListNode>()), address);
        assert!(size >= mem::size_of::<ListNode>());

        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = address as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    // Looks for a free region with the given size and alignment and removes it from the list
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        // refernce to current list node, updated for each iteration
        let mut current = &mut self.head;
        // look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // region suitable for allocation -> remove node from list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            }

            // region not suitable for allocation -> continue with next region
            current = current.next.as_mut().unwrap()
        }

        None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_address(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_address() {
            // region too small
            return Err(());
        }

        let excess_size = region.end_address() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of the region too small to hold a ListNode (required because the allocation
            // splits the region in a used and a free part
            return Err(());
        }
        
        // region suitable for allocation
        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_address() - alloc_end;
            
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }

            return alloc_start as *mut u8;
        }

        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
