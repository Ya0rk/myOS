mod context;
mod user_trap;
mod kernel_trap;

use alloc::sync::Arc;
use user_trap::{user_trap_handler, user_trap_return};
use core::arch::global_asm;
use core::fmt::Display;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;
use crate::signal::do_signal;
use crate::sync::{get_waker, suspend_now};
use crate::task::{TaskControlBlock, TaskStatus};
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

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
    loop {
        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {},
        }

        user_trap_return();

        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {},
        }

        user_trap_handler().await;

        match task.get_status() {
            TaskStatus::Zombie => break,
            TaskStatus::Stopped => suspend_now().await,
            _ => {},
        }

        if task.pending() {
            do_signal(&task);
        }
    }

    task.exit();
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

fn set_trap_handler(mode: IndertifyMode) {
    match mode {
        IndertifyMode::User => {
            unsafe {
                stvec::write(__trap_from_user as usize, TrapMode::Direct);
            }
        },
        IndertifyMode::Kernel => {
            unsafe {
                stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
            }
        },
    }
}