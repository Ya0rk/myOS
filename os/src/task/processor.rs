use core::cell::UnsafeCell;
use super::TaskControlBlock;
use crate::config::HART_NUM;
use crate::mm::page_table::switch_to_kernel_pgtable;
// use crate::mm::switch_to_kernel_pgtable;
use crate::sync::disable_interrupt;
use crate::sync::enable_interrupt;
use crate::trap::TrapContext;
use crate::utils::backtrace;
use alloc::sync::Arc;

///CPU 结构体，包含当前正在运行的任务和内核线程的上下文
pub struct CPU {
    current: Option<Arc<TaskControlBlock>>,
    hart_id: usize,
}

impl CPU {
    pub const fn new() -> Self {
        Self {
            current: None,
            hart_id: 0,
        }
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }

    /// 将当前任务设置为None
    pub fn clear_cpu_task(&mut self) {
        self.current = None;
    }
    /// 设置当前任务
    pub fn set_cpu_task(&mut self, task: Arc<TaskControlBlock>) {
        self.current = Some(task);
    }

}

impl CPU {
    /// 将task装载进处理器
    pub fn user_task_checkin(&mut self, task: &mut Arc<TaskControlBlock>) {
        disable_interrupt();
        //TODO:完善TIME_STAT
        task.get_time_data_mut().set_sched_in_time();
        self.set_cpu_task(task.clone());
        task.switch_pgtable();
        enable_interrupt();
    }

    /// 将当前任务从处理器中取出，为下一个task让出处理器
    pub fn user_task_checkout(&mut self, task: &mut Arc<TaskControlBlock>) {
        disable_interrupt();
        // TODO:完善TIME_STAT
        // 实现float reg的保存
        switch_to_kernel_pgtable();
        current_trap_cx().float_regs.sched_out_do_with_freg();
        self.clear_cpu_task();
        task.get_time_data_mut().set_sched_out_time();
        enable_interrupt();
    }
}

pub struct SyncProcessors(UnsafeCell<[CPU; HART_NUM]>);
unsafe impl Sync for SyncProcessors {}

/// 多核管理器 TODO
/// 每个核只会访对应id的Processor，所以不需要加锁
const PROCESSOR: CPU = CPU::new();
pub static PROCESSORS: SyncProcessors = SyncProcessors(UnsafeCell::new([PROCESSOR; HART_NUM]));


///Init PROCESSORS
pub fn init_processors() {
    unsafe {
        for (id, p) in (&mut *PROCESSORS.0.get()).iter_mut().enumerate() {
            p.hart_id = id;
        }
    }
    println!("procs init successfully!");
}

/// 获取当前运行的 Processor
pub fn get_current_cpu() -> &'static mut CPU {
    let id = get_current_hart_id();
    unsafe { &mut (*PROCESSORS.0.get())[id] }
}

#[allow(unused)]
/// 根据 hart_id 获取对应的 Processor
pub fn get_cpu(hart_id: usize) -> &'static mut CPU {
    unsafe { &mut (*PROCESSORS.0.get())[hart_id] }
}

/// 获取当前运行的 Processor id
pub fn get_current_hart_id() -> usize {
    // unimplemented!()
    // use core::arch::asm;
    // let hartid;
    // unsafe {
    //     asm! {
    //         "mv {}, tp",
    //         out(reg) hartid
    //     };
    // }
    // hartid
    crate::arch::tp_read()
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
// pub fn run_tasks() {
//     loop {
//         let processor = get_current_processor();
//         if let Some(task) = fetch_task() {
//             let idle_task_cx_ptr = processor.get_idle_task_cx_ptr(); // 内核线程负责分发用户线程
//             // access coming task TCB exclusively
//             let next_task_cx_ptr = task.get_task_cx() as *const TaskContext;
//             task.set_running();
//             task.switch_pgtable(); // 更新stap寄存器和刷新TLB
            
//             // release coming task TCB manually
//             processor.current = Some(task);
//             unsafe {
//                 __switch(idle_task_cx_ptr, next_task_cx_ptr);
//             }
//             unsafe { KERNEL_SPACE.lock().activate() };
//         }
//     }
// }

///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    get_current_cpu().take_current()
}

/// 获取当前正在运行的任务
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    get_current_cpu().current()
}

///Get token of the address space of current task
pub fn current_user_token() -> usize {
    // riscv::register::satp::read().bits()
    // unimplemented!()
    crate::arch::satp_read()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    if current_task().is_none() {
        backtrace();
    }
    current_task()
        .unwrap()
        .get_trap_cx_mut()
}

// /Return to idle control flow for new scheduling
// pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
//     let processor = get_current_processor();
//     let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
//     unsafe {
//         __switch(switched_task_cx_ptr, idle_task_cx_ptr);
//     }
// }