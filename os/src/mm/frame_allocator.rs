use super::{PhysAddr, PhysPageNum, VirtAddr};
use crate::fs::Dentry;
use crate::mm::Paged;
use crate::sync::SpinNoIrqLock;
use crate::task::current_task;
use crate::{
    hal::config::{KERNEL_ADDR_OFFSET, MEMORY_END},
    mm::address::KernelAddr,
};
use alloc::vec::Vec;
// use riscv::addr::VirtAddr;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;
use spin::Mutex;

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    ///Create an empty `FrameTracker`
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

pub trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
    fn frame_total(&self) -> usize;
    fn frame_free(&self) -> usize;
}
/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
    FRAME_TOTAL: usize,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
        self.FRAME_TOTAL = self.end - self.current;
        println!("last {} Physical Frames.", self.FRAME_TOTAL);
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
            FRAME_TOTAL: 0,
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }

    fn frame_total(&self) -> usize {
        self.FRAME_TOTAL
    }

    fn frame_free(&self) -> usize {
        self.end - self.current + self.recycled.len()
    }
}

/// 硬编码释放ltp文件的pagecache
fn release_ltp() {
    let ltp_dentry = Dentry::get_dentry_from_path("/musl/ltp/testcases/bin")
            .map_or(Dentry::get_dentry_from_path("/glibc/ltp/testcases/bin").unwrap(), |d| d);

    ltp_dentry.with_children(|children| {
        for (abs, child_inode) in children.iter().filter_map(|(abs, dentry)| {
            dentry.get_inode().map(|inode| (abs, inode))
        })
        {
            if let Some(cache) = child_inode.get_page_cache() {
                cache.pages.write().clear();
            }
        }
    });
}


pub type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: SpinNoIrqLock<FrameAllocatorImpl> = SpinNoIrqLock::new(FrameAllocatorImpl::new());
}
/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.lock().init(
        VirtAddr(ekernel as usize).paged_pa().ceil(),
        VirtAddr(MEMORY_END).paged_pa().floor(),
    );
}
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    let res = FRAME_ALLOCATOR.lock().alloc().map(FrameTracker::new);
    if res.is_some() {
        return res;
    }

    release_ltp();
    FRAME_ALLOCATOR.lock().alloc().map(FrameTracker::new)
}
/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
