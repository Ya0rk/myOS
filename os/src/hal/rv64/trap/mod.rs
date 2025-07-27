pub mod context;
pub mod kernel_trap;
pub mod user_trap;

use crate::hal::arch::shutdown;
use crate::signal::do_signal;
use crate::sync::{get_waker, suspend_now};
use crate::task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
pub use context::TrapContext;
pub use context::UserFloatRegs;
use core::arch::global_asm;
use core::fmt::Display;
use log::info;
use user_trap::{user_trap_handler, user_trap_return};
use crate::hal::arch::shutdown;
use crate::signal::do_signal;
use crate::sync::{get_waker, suspend_now};
use crate::task::{TaskControlBlock, TaskStatus};
pub use context::TrapContext;
pub use context::UserFloatRegs;
// riscv架构有关
#[cfg(target_arch = "riscv64")]
use riscv::register::mtvec::TrapMode;
#[cfg(target_arch = "riscv64")]
use riscv::register::stvec;
// loongarch架构有关
#[cfg(target_arch = "loongarch64")]
use loongarch64::register::ecfg::LineBasedInterrupt;
#[cfg(target_arch = "loongarch64")]
use loongarch64::register::estat::{Exception, Interrupt, Trap};
#[cfg(target_arch = "loongarch64")]
use loongarch64::register::*;
#[cfg(target_arch = "loongarch64")]
use loongarch64::time::get_timer_freq;

// trap汇编代码
#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("trap.S"));
#[cfg(target_arch = "loongarch64")]
global_asm!(include_str!("la64_trap.S"));

// 申明外部函数，这些函数是在汇编代码中实现的，用于从用户态和内核态切换
extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
    pub fn __sigret_helper();
    #[allow(improper_ctypes)]
    fn __return_to_user(ctx: *mut TrapContext);
}

pub fn init() {
    set_trap_handler(IndertifyMode::Kernel);
}

/// 用户态陷入内核态后，执行完内核态代码后，返回用户态
pub async fn trap_loop(task: Arc<TaskControlBlock>) {
    // 设置task的waker TODO：将这个放入 UserTaskFuture中
    task.set_task_waker(get_waker().await);
    info!("trap loop!!");
    loop {
        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        // debug_point!("before enable sext");
        unsafe {
            crate::hal::rv64::arch::interrupt::device_init();
        };
        // debug_point!("after enable sext");

        user_trap_return();

        unsafe {
            crate::hal::rv64::arch::interrupt::disenable_supervisor_extern_interrupt();
        }

        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        user_trap_handler().await;

        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {}
        }

        if task.pending() {
            do_signal(&task);
        }
    }
    info!("[trap loop] task pid = {} exit", task.get_pid());
    task.check_shutdown();
    task.do_exit();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
enum IndertifyMode {
    User,
    Kernel,
}

impl Display for IndertifyMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IndertifyMode::User => write!(f, "User"),
            IndertifyMode::Kernel => write!(f, "Kernel"),
        }
    }
}

#[cfg(target_arch = "riscv64")]
fn set_trap_handler(mode: IndertifyMode) {
    // unimplemented!()
    match mode {
        IndertifyMode::User => unsafe {
            stvec::write(__trap_from_user as usize, TrapMode::Direct);
        },
        IndertifyMode::Kernel => unsafe {
            stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
        },
    }
}
/// 应当注意到这个函数的功能并不正确，__trap_from_user应当为虚拟页号而不是函数指针。
/// 需要去修改链接器，保证其为页面对齐的
/// 信息来自loongarch手册关于csr eentry
#[cfg(target_arch = "loongarch64")]
#[inline]
fn set_trap_handler(mode: IndertifyMode) {
    match mode {
        IndertifyMode::User => unsafe {
            ecfg::set_vs(0);
            eentry::set_eentry(__trap_from_user as usize);
        },
        IndertifyMode::Kernel => unsafe {
            ecfg::set_vs(0);
            eentry::set_eentry(__trap_from_kernel as usize);
        },
    }
}

#[cfg(target_arch = "loongarch64")]
pub fn init_loongarch() {}
