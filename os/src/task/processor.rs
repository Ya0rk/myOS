use core::cell::UnsafeCell;
use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::HART_NUM;
use crate::mm::KERNEL_SPACE;
use crate::trap::TrapContext;
use crate::utils::backtrace;
use alloc::sync::Arc;
use alloc::boxed::Box;

///CPU 结构体，包含当前正在运行的任务和内核线程的上下文
pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: Option<Box<TaskContext>>,
    hart_id: usize,
}

impl Processor {
    pub const fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: None,
            hart_id: 0,
        }
    }
    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        self.idle_task_cx.as_mut().unwrap().as_mut() as *mut _
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    pub fn get_hart_id(&self) -> usize {
        self.hart_id
    }
    pub fn set_hart_id(&mut self, hart_id: usize) {
        self.hart_id = hart_id;
    }
}

pub struct SyncProcessors(UnsafeCell<[Processor; HART_NUM]>);
unsafe impl Sync for SyncProcessors {}

/// 多核管理器 TODO
/// 每个核只会访对应id的Processor，所以不需要加锁
const PROCESSOR: Processor = Processor::new();
pub static PROCESSORS: SyncProcessors = SyncProcessors(UnsafeCell::new([PROCESSOR; HART_NUM]));


///Init PROCESSORS
pub fn init_processors() {
    unsafe {
        for (id, p) in (&mut *PROCESSORS.0.get()).iter_mut().enumerate() {
            p.idle_task_cx = Some(Box::new(TaskContext::zero_init()));
            p.hart_id = id;
        }
    }
    println!("procs init successfully!");
}

pub fn get_proc_by_hartid(hartid: usize) -> &'static mut Processor {
    unsafe { &mut (*PROCESSORS.0.get())[hartid] }
}

/// 获取当前运行的 CPU 核
pub fn get_current_hart_id() -> usize {
    use core::arch::asm;
    let hartid;
    unsafe {
        asm! {
            "mv {}, tp",
            out(reg) hartid
        };
    }
    hartid
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let hart_id = get_current_hart_id();
        let processor = get_proc_by_hartid(hart_id);
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr(); // 内核线程负责分发用户线程
            // access coming task TCB exclusively
            let mut task_inner = task.inner_lock();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            unsafe { task_inner.memory_set.activate() }; // 更新stap寄存器和刷新TLB
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
            unsafe { KERNEL_SPACE.lock().activate() };
        }
    }
}
///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    get_proc_by_hartid(get_current_hart_id()).take_current()
}
///Get running task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    get_proc_by_hartid(get_current_hart_id()).current()
}
///Get token of the address space of current task
pub fn current_user_token() -> usize {
    riscv::register::satp::read().bits()
}
///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    if current_task().is_none() {
        backtrace();
    }
    current_task()
        .unwrap()
        .inner_lock()
        .get_trap_cx()
}
///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let processor = get_proc_by_hartid(get_current_hart_id());
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
