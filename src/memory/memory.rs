// Page Table format
//
// Page tables need to be page-aligned, aligned on a 4 KiB boundary.
// This requirement guarantees that a page table always fills a complete page and allows an
// optimization that makes entries very compact
//
// Each entry is 8 bytes large and has the following format:
//
// Bit(s) Name                   Meaning
// 0      present                the page is currently in memory
// 1      writable               it's allowed to write to this page
// 2      user accessible        if not set, only kernel mode code can access this page
// 3      write-through caching  writes go directly to memory
// 4      disable cache          no cache is used fro this page
// 5      accessed               the CPU sets this bit when this page is used
// 6      dirty                  the CPU sets this bit when a write to this page occurs
// 7      huge page/null         must be 0 in P1 and P4, creates a 1 GiB page in P3, creates a 2
                             //  MiB page in P2
// 8      global                 page isn't flushed from caches on address space switch(PGE bit of
                             //  CR4 register must be set)
// 9-11   available              can be used freely by the OS
// 12-51  physical address       the page aligned 52bit physical address of the frame or the next
                             //  page table
// 52-62  available              can be used freely by the OS
// 63     no execute             forbid executing code on this page(the NXE bit in this EFER
                            //   register must be set
//
// Virtual Address (x86_64)
//
// 00000000 00000000 000 000 001 000 000 000 111 111 111 001 111 111 0101 1100 1110
// ----------------- ----------- ----------- ----------- ----------- --------------
// Sign Extension    Level 4     Level 3     Level 2     Level 1     Offset
//                   Index = 1   Index = 0   Index = 511 Index = 127 = 0x5CF
//
//! KEEP THINGS TO THE SAME SIZE

use x86_64:: {
    structures::paging::{PageTable, OffsetPageTable},
    VirtAddr
};

// Returns a mutable reference to the active level 4 table.
//
// return value is unsafe because the caller must guarantee that the
// complete physical memory is mapped to virtual memory at the passed
// `physical_memory_offset`. Also, this function mst be only called once
// to avoid aliasing `&mut` references (which is undefined behavior).
pub fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level4_table = active_level4_table(physical_memory_offset);
    unsafe { OffsetPageTable::new(level4_table, physical_memory_offset) }
}
fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level4_table_frame, _) = Cr3::read();

    let physical_address = level4_table_frame.start_address();
    let virtual_address = physical_memory_offset + physical_address.as_u64();
    let page_table_ptr: *mut PageTable = virtual_address.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

use x86_64::{
    PhysAddr,
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
};

pub fn create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // TODO
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush()
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

// A FrameAllocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
impl BootInfoFrameAllocator {
    // Create BootInfoFrameAllocator from the passed memory map.
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let address_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an itterator of frame from start addresses
        let frame_addresses = address_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
