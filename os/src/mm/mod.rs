pub mod address;
mod frame_allocator;
mod heap_allocator;
// mod memory_set;
pub mod page_table;
mod ffi;
mod userbuffer;
// mod map_area;
pub mod memory_space;
pub mod page;
// mod user_ptr;
// mod addr;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, KernelAddr, StepByOne};
use alloc::sync::Arc;
use spin::Mutex;
pub use frame_allocator::FrameTracker;
// pub use memory_set::{MemorySet, KERNEL_SPACE};
pub use ffi::{MapPermission, MapType};
use page_table::{switch_to_kernel_pgtable, KERNEL_PAGE_TABLE};
pub use userbuffer::UserBuffer;
// pub use map_area::MapArea;
pub use page_table::{PageTable};
pub use frame_allocator::{frame_alloc, frame_dealloc};
// pub use memory_set::{remap_test, kernel_token, switch_to_kernel_pgtable};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_ref, translated_str};

/// initiate heap allocator, frame allocator and kernel space
pub fn init(first: bool) {
    if first {
        heap_allocator::init_heap();
        frame_allocator::init_frame_allocator();
    }
    // unsafe {
    //     KERNEL_PAGE_TABLE = Some(Arc::new(Mutex::new(PageTable::init_kernel_page_table())));
    // }
    #[cfg(target_arch = "riscv64")]
    switch_to_kernel_pgtable(); // 切换到kernel page table
}
