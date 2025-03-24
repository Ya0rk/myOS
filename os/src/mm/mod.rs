mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;
mod ffi;
mod userbuffer;
mod map_area;
mod memory_space;
mod page;
mod user_ptr;
// mod addr;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, KernelAddr, StepByOne};
pub use frame_allocator::FrameTracker;
pub use memory_set::{MemorySet, KERNEL_SPACE};
pub use ffi::{MapPermission, MapType};
pub use userbuffer::UserBuffer;
pub use map_area::MapArea;
pub use page_table::{PageTableEntry, PageTable};
pub use frame_allocator::{frame_alloc, frame_dealloc};
pub use memory_set::{remap_test, kernel_token};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_ref, translated_str};

/// initiate heap allocator, frame allocator and kernel space
pub fn init(first: bool) {
    if first {
        heap_allocator::init_heap();
        frame_allocator::init_frame_allocator();
    }
    unsafe { KERNEL_SPACE.lock().activate() }; // 切换到kernel page table
}
