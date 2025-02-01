mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, KernelAddr, StepByOne};
pub use frame_allocator::FrameTracker;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{PageTableEntry, PageTable, UserBuffer};
pub use frame_allocator::{frame_alloc, frame_dealloc};
pub use memory_set::{remap_test, kernel_token};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str};

use page_table::PTEFlags;
use address::VPNRange;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
