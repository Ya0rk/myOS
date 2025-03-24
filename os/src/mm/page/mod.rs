use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use alloc::sync::Arc;
use crate::fs::File;
use crate::mm::address::PhysPageNum;
pub struct Page {
    frame: FrameTracker,
    page_type: PageType,
}

pub enum PageType {
    Anon,
    File(Arc<dyn File>),
}


impl Page {
    pub fn fill_zero(&self) {
        unimplemented!()
    }

    pub fn new() -> Arc<Self> {
        let frame = frame_alloc().expect("no frame available");
        Arc::new(Self {
            frame,
            page_type: PageType::Anon,
        })
    }

    pub fn copy_from_page(&self, other: &Self) {
        unimplemented!()
    }

    pub fn ppn(&self) -> PhysPageNum {
        self.frame.ppn
    }

}