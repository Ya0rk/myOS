use core::{future::Future, pin::Pin};
use alloc::sync::Arc;

use crate::trap::trap_loop;

use super::{executor, processor::get_current_processor, TaskControlBlock};


pub struct UserTaskFuture<F: Future + Send + 'static> {
    task: Arc<TaskControlBlock>,
    future: F,
}

impl <F: Future + Send + 'static> UserTaskFuture<F> {
    fn new(task: Arc<TaskControlBlock>, future: F) -> Self {
        Self {
            task,
            future,
        }
    }
}

impl<F: Future + Send + 'static> Future for UserTaskFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let processor = get_current_processor();
        processor.user_task_checkin(&mut this.task); // 将任务装载进处理器
        let ret = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };
        processor.user_task_checkout(&mut this.task); // 将任务从处理器中取出
        ret
    }
}


pub struct KernelTaskFuture<F: Future + Send + 'static> {
    future: F,
}

impl <F: Future + Send + 'static> KernelTaskFuture<F> {
    fn new(future: F) -> Self {
        Self {
            future,
        }
    }
}

impl<F: Future + Send + 'static> Future for KernelTaskFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        // TODO: 实现kernel中断时完善 checkin checkout
        let ret = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };
        ret
    }
}

/// 用于设置用户态任务
pub fn spawn_user_task(user_task: Arc<TaskControlBlock>) {
    let future = UserTaskFuture::new(user_task.clone(), trap_loop(user_task));
    executor::spawn(future);
}

/// 用于设置定时任务和initproc
pub fn spawn_kernel_task<T: Future<Output = ()> + Send + 'static>(kernel_task: T) {
    let future = KernelTaskFuture::new(kernel_task);
    executor::spawn(future);
}