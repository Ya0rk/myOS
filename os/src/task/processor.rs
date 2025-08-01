use super::TaskControlBlock;
use crate::hal::config::HART_NUM;
use crate::mm::page_table::enable_kernel_pgtable;
use crate::utils::SysResult;
use core::cell::UnsafeCell;
// use crate::mm::switch_to_kernel_pgtable;
use crate::hal::trap::TrapContext;
use crate::sync::disable_supervisor_interrupt;
use crate::sync::enable_supervisor_interrupt;
use crate::utils::backtrace;
use alloc::sync::Arc;

///CPU 结构体，包含当前正在运行的任务和内核线程的上下文
pub struct CPU {
    current: Option<Arc<TaskControlBlock>>,
    /// 计数器，记录内核时钟中断的次数
    /// 在内核时钟中断处理函数中会增加这个计数器
    /// 在trap return时会清零
    /// 这个计数器的作用是为了在内核线程中进行调度
    /// 如果计数器大于一定值，就yield进行调度
    /// 避免该线程一直占用CPU
    timer_irq_cnt: usize,
    hart_id: usize,

    // k_int_cnt: usize,
    // k_int_mask: bool,
    /// return value of kernel trap
    kernel_trap_ret_value: Option<SysResult<()>>,
    // 模拟寄存器传参，改为使用全局变量实现
    // 使用参数必须关中断
    // kernel_trap_arg0: Option<usize>,
    // kernel_trap_arg1: Option<usize>,
}

impl CPU {
    pub const fn new() -> Self {
        Self {
            current: None,
            timer_irq_cnt: 0,
            hart_id: 0,
            kernel_trap_ret_value: None,
            // kernel_trap_arg0: None,
            // kernel_trap_arg1: None,
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
    pub fn timer_irq_inc(&mut self) {
        self.timer_irq_cnt += 1;
    }
    pub fn timer_irq_cnt(&self) -> usize {
        self.timer_irq_cnt
    }
    pub fn timer_irq_reset(&mut self) {
        self.timer_irq_cnt = 0;
    }

    pub fn take_ktrap_ret(&mut self) -> Option<SysResult<()>> {
        self.kernel_trap_ret_value.take()
    }

    pub fn set_ktrap_ret(&mut self, ret: SysResult<()>) {
        self.kernel_trap_ret_value = Some(ret);
    }

    // pub fn set_ktrap_arg0(&mut self, arg0: usize) {
    //     self.kernel_trap_arg0 = Some(arg0);
    // }
    // pub fn set_ktrap_arg1(&mut self, arg1: usize) {
    //     self.kernel_trap_arg1 = Some(arg1);
    // }
    // pub fn take_ktrap_arg0(&mut self) -> Option<usize> {
    //     self.kernel_trap_arg0.take()
    // }
    // pub fn take_ktrap_arg1(&mut self) -> Option<usize> {
    //     self.kernel_trap_arg1.take()
    // }
}

impl CPU {
    /// 将task装载进处理器
    pub fn user_task_checkin(&mut self, task: &mut Arc<TaskControlBlock>) {
        disable_supervisor_interrupt();
        //TODO:完善TIME_STAT
        task.get_time_data_mut().set_sched_in_time();
        self.set_cpu_task(task.clone());
        task.switch_pgtable();
        // enable_interrupt();
    }

    /// 将当前任务从处理器中取出，为下一个task让出处理器
    pub fn user_task_checkout(&mut self, task: &mut Arc<TaskControlBlock>) {
        // disable_interrupt();
        // TODO:完善TIME_STAT
        // 实现float reg的保存
        // enable_kernel_pgtable();
        current_trap_cx().float_regs.sched_out_do_with_freg();
        self.clear_cpu_task();
        task.get_time_data_mut().set_sched_out_time();
        enable_supervisor_interrupt();
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
    crate::hal::arch::tp_read()
}

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
    crate::hal::arch::satp_read()
}

///Get token of the address space of kernel
pub fn current_kernel_token() -> usize {
    // riscv::register::satp::read().bits()
    // unimplemented!()
    crate::hal::arch::kernel_token_read()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    if current_task().is_none() {
        backtrace();
    }
    current_task().unwrap().get_trap_cx_mut()
}

#[inline(always)]
pub fn set_ktrap_ret(ret: SysResult<()>) {
    get_current_cpu().set_ktrap_ret(ret);
}

#[inline(always)]
pub fn take_ktrap_ret() -> Option<SysResult<()>> {
    get_current_cpu().take_ktrap_ret()
}
