use core::fmt;

use crate::hal::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, INITPROC_PID};
use crate::mm::page_table::{ KERNEL_PAGE_TABLE};
use crate::hal::mem::page_table::PTEFlags;
use crate::mm::{VirtAddr};
use alloc::collections::BTreeSet;
use lazy_static::*;
use log::{debug, info};
// use riscv::paging::PTE;
use spin::Mutex;

lazy_static! {
    pub static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
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
        assert!(
            self.recycled.insert(pid), // 插入，如果失败说明 PID 已经回收过
            "pid {} has been deallocated!",
            pid
        );
    }
}

///Bind pid lifetime to `PidHandle`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl Drop for Pid {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}
///Allocate a pid from PID_ALLOCATOR
pub fn pid_alloc() -> Pid {
    PID_ALLOCATOR.lock().alloc()
}



// TODO:实现了无栈协程后就不需要内核栈了，后面要删除

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE); // 这里的page_size是为了隔离不同app的kernel stack，stack guard
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

///Kernelstack for app
pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    ///每个进程有不同的pid，根据pid来分配的kernel stack位置也不同
    pub fn new(pid_handle: &Pid) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        debug!("kernel stack bottom: {:#x}, top: {:#x}", 
                kernel_stack_bottom, 
                kernel_stack_top
            );
        let flags = PTEFlags::V | PTEFlags::R | PTEFlags::W;
        KERNEL_PAGE_TABLE.lock().map_kernel_range(kernel_stack_bottom.into()..kernel_stack_top.into(), flags);
        KernelStack { pid: pid_handle.0 }
    }
    #[allow(unused)]
    ///Push a value on top of kernelstack
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where
        T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }
    /// 获取到kernek stack的top，高地址
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
    /// 获取到kernek stack的bottom，低地址
    pub fn get_bottom(&self) -> usize {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        kernel_stack_bottom
    }
    /// 获取到kernek stack的bottom和top
    pub fn get_kernel_stack_pos(&self) -> (usize, usize) {
        kernel_stack_position(self.pid)
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, kernel_stack_start) = kernel_stack_position(self.pid);
        // let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        // KERNEL_SPACE
        //     .lock()
        //     .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
        info!("[KernelStack] Drop for pid {}", self.pid);
        KERNEL_PAGE_TABLE.lock().unmap_kernel_range(kernel_stack_bottom.into()..kernel_stack_start.into());
    }
}
