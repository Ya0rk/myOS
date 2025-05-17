use core::fmt;

use crate::hal::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, INITPROC_PID};
use crate::mm::memory_space::vm_area::MapPerm;
use crate::mm::page_table::{ KERNEL_PAGE_TABLE};
use crate::hal::mem::page_table::PTEFlags;
use crate::mm::{VirtAddr};
use crate::sync::SpinNoIrqLock;
use alloc::collections::BTreeSet;
use lazy_static::*;
use log::{debug, info};
// use riscv::paging::PTE;
use spin::Mutex;
// use riscv::paging::PTE;

lazy_static! {
    pub static ref PID_ALLOCATOR: SpinNoIrqLock<PidAllocator> = SpinNoIrqLock::new(PidAllocator::new());
}
///分配和管理pid号，避免重复
pub struct PidAllocator {
    current: usize,
    recycled: BTreeSet<usize>,
}

impl PidAllocator {
    fn new() -> Self {
        PidAllocator {
            current: INITPROC_PID,
            recycled: BTreeSet::new(),
        }
    }
    /// 分配一个pid
    fn alloc(&mut self) -> Pid {
        if let Some(pid) = self.recycled.pop_first() {
            Pid(pid)
        } else {
            let pid = self.current;
            self.current += 1;
            Pid(pid)
        }
    }
    /// 删除一个pid，放入recycled中
    fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current, "pid {} is out of range", pid);
        if pid == 8 {
            info!("pid 8 dealloc");
        }
        assert!(
            self.recycled.insert(pid), // 插入，如果失败说明 PID 已经回收过
            "pid {} has been deallocated!",
            pid
        );
    }
}

///Bind pid lifetime to `PidHandle`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pid(pub usize);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Pid> for usize {
    fn from(value: Pid) -> Self {
        value.0
    }
}

impl From<usize> for Pid {
    fn from(value: usize) -> Self {
        Pid(value)
    }
}

impl Drop for Pid {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}
///Allocate a pid from PID_ALLOCATOR
pub fn pid_alloc() -> Pid {
    PID_ALLOCATOR.lock().alloc()
}
