
mod heap_allocator;
pub mod address;
pub mod memory_space;
pub mod page;
pub mod page_table;
pub mod user_ptr;
pub mod frame_allocator;

use alloc::sync::Arc;
pub use frame_allocator::FrameTracker;
use spin::Mutex;
// pub use memory_set::{MemorySet, KERNEL_SPACE};
pub use address::{
    Direct, KernelAddr, PageNum, Paged, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum,
};
use page_table::{enable_kernel_pgtable, KERNEL_PAGE_TABLE};
// pub use userbuffer::UserBuffer;
// pub use map_area::MapArea;
pub use frame_allocator::{frame_alloc, frame_dealloc};
pub use page_table::PageTable;
// pub use page_table::{translated_byte_buffer, translated_ref, translated_refmut, translated_str};

/// initiate heap allocator, frame allocator and kernel space
pub fn init(first: bool) {
    if first {
        heap_allocator::init_heap();
        frame_allocator::init_frame_allocator();
    }
    // unsafe {
    //     KERNEL_PAGE_TABLE = Some(Arc::new(Mutex::new(PageTable::init_kernel_page_table())));
    // }
    // #[cfg(target_arch = "riscv64")]
    enable_kernel_pgtable(); // 切换到kernel page table
}
