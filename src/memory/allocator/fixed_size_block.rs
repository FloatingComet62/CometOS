use core::{
    alloc::Layout,
    ptr::{
        self,
        NonNull,
    },
    mem,
};
use super::Locked;
use alloc::alloc::GlobalAlloc;

struct ListNode {
    next: Option<&'static mut ListNode>,
}

// The block sizes to use
//
/// The sizes must be a power of 2
// because they are also used as the block alignment
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}
impl FixedSizeBlockAllocator {
    // Creates an empty FixedSizeBlockAllocator
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty()
        }
    }

    // Initialise the allocator with the given heap bounds
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    // Allocates using the fallback allocator
    fn fallback_allocator(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

// Choose an appropriate block size for the given layout
// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    // First, we use the Locked::lock method to get a mutable reference to the wrapped allocator
    // instance. Next, we call the list_index function we just defined to calculate the appropriate
    //
    // block size for the give layout and get the corresponding index into the list_heads array. If
    // this index is None, no block size fits for the allocation, we use the fallback_allocator
    //
    // If the list index is Some, we try to remove the first node in the corresponding list started
    // by list_heads[index] using the Option::take method. If the list is not empty, we enter the
    // Some(node) branch of the match statement, where we point the head pointer of the list to
    // the successor of the popped node (by using take again).
    // Finally we return the popped node pointer as *mut u8.
    //
    // If the list head is None, it indicates that the list of blocks is empty. This means that we
    // need to construct a new block. For that, we first get the current block size from the
    // BLOCK_SIZES slice and use it as both the size and the alignment for the new block. Then we
    // create a new Layout from it and call the fallback_allocator method to performe the
    // allocation. The reason for adjusting the layout and alignment is that the block will be
    // added to the block list on deallocation.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are a power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_allocator(layout)
                    }
                }
            }
            None => allocator.fallback_allocator(layout),
        }
    }

    // Like in alloc, we first use the lock method to get a mutable allocator reference
    // and then the list_index function to get the block list corresponding to the given Layout. If
    // the index is None, no fitting block size exist in BLOCK_SIZES, which indicates that the
    // allocation was created by the fallback allocator. Therefor, we use it's deallocate to free
    // the memory again. The method expects a NonNull instead of a *mut u8, so we need to convert
    // the pointer first. (The unwrap call only fails when the pointer is null, which should never
    // happen when the compiler calls dealloc.)
    //
    // If list_index returns a block index, we need to add the freed memory block to the list. For
    // that, we first create a new ListNode that points to the current list head (by using
    // Option::take again). Before we write the new node into the freed memory block, we first
    // assert the current block size specified by index has the required size and alignment for
    // storing a ListNode. Then we perform the write by converting the given *mut u8 pointer to a
    // *mut ListNode pointer and then calling the unsafe write method on it. The last step is to
    // set the head pointer of the list, which is currently None since we called take on it, to our
    // newly written ListNode. For that, we convert the raw_node_ptr to a mutable reference.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take()
                };
                // verify that block has size and alignment required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}
