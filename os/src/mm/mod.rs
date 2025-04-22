mod frame_allocator;
mod heap_allocator;
mod ffi;
mod userbuffer;
mod map_area;
pub mod memory_space;
pub mod page;
pub mod page_table;
pub mod address;
pub mod user_ptr;

use alloc::sync::Arc;
use spin::Mutex;
use page_table::{switch_to_kernel_pgtable, KERNEL_PAGE_TABLE};
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, KernelAddr, StepByOne};
pub use frame_allocator::FrameTracker;
pub use ffi::{MapPermission, MapType};
pub use userbuffer::UserBuffer;
pub use map_area::MapArea;
pub use page_table::{PageTableEntry, PageTable};
pub use frame_allocator::{frame_alloc, frame_dealloc};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_ref, translated_str};

/// initiate heap allocator, frame allocator and kernel space
pub fn init(first: bool) {
    if first {
        heap_allocator::init_heap();
        frame_allocator::init_frame_allocator();
    }
    switch_to_kernel_pgtable(); // 切换到kernel page table
}
