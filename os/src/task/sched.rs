use core::{future::Future, pin::Pin};
use alloc::sync::Arc;
use crate::hal::trap::trap_loop;
use super::{executor, processor::get_current_cpu, TaskControlBlock};

pub enum TaskFuture<F: Future<Output=()> + Send + 'static> {
    UserTaskFuture {
        task: Arc<TaskControlBlock>,
        future: F,
    },
    KernelTaskFuture {
        future: F,
    },
}

impl<F: Future<Output=()> + Send + 'static> TaskFuture<F> {
    /// 创建一个用户任务的 Future
    pub fn user_task(task: Arc<TaskControlBlock>, future: F) -> Self {
        TaskFuture::UserTaskFuture { task, future }
    }

    /// 创建一个内核任务的 Future
    pub fn kernel_task(future: F) -> Self {
        TaskFuture::KernelTaskFuture { future }
    }
}


impl<F: Future<Output=()> + Send + 'static> Future for TaskFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        match this {
            TaskFuture::UserTaskFuture { task, future } => {
                let processor = get_current_cpu();
                processor.user_task_checkin(task); // 用户任务 checkin
                let ret = unsafe { Pin::new_unchecked(future).poll(cx) };
                processor.user_task_checkout(task); // 用户任务 checkout
                ret
            }
            TaskFuture::KernelTaskFuture { future } => {
                // TODO: 实现kernel中断时完善 checkin checkout
                unsafe { Pin::new_unchecked(future).poll(cx) }
            }
        }
    }
}

/// 用于设置用户态任务
pub fn spawn_user_task(user_task: Arc<TaskControlBlock>) {
    let future = TaskFuture::user_task(user_task.clone(), trap_loop(user_task));
    executor::spawn(future);
}

/// 用于设置定时任务和initproc
pub fn spawn_kernel_task<T: Future<Output = ()> + Send + 'static>(kernel_task: T) {
    let future = TaskFuture::kernel_task(kernel_task);
    executor::spawn(future);
}