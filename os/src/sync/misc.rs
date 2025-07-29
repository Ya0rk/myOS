#![allow(unused)]

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::sync::Arc;
use log::info;

use crate::{signal::SigMask, task::TaskControlBlock};

struct GetWakerFuture;

impl Future for GetWakerFuture {
    type Output = Waker;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        core::task::Poll::Ready(cx.waker().clone())
    }
}

/// 获取当前任务的waker
#[inline(always)]
pub async fn get_waker() -> Waker {
    GetWakerFuture.await
}

enum ControlBehavior {
    YieldFuture,
    SuspendFuture,
}

struct ControlFuture {
    behavior: ControlBehavior,
    is_done: bool,
}

impl ControlFuture {
    fn new(behavior: ControlBehavior) -> Self {
        Self {
            behavior,
            is_done: false,
        }
    }
}

impl Future for ControlFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.is_done {
            true => Poll::Ready(()),
            false => {
                self.is_done = true;
                match &self.behavior {
                    ControlBehavior::YieldFuture => {
                        // 让出当前任务：唤醒自己，重新加入任务队列
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    ControlBehavior::SuspendFuture => {
                        // 挂起当前任务：不唤醒自己，等待其他任务唤醒
                        Poll::Pending
                    }
                }
            }
        }
    }
}

/// 放弃当前任务的执行，将其重新加入task_queue中轮循，使得其他任务有机会执行
#[inline(always)]
pub async fn yield_now() {
    // info!("yield now");
    ControlFuture::new(ControlBehavior::YieldFuture).await
}

/// 挂起当前任务，使得其他任务有机会执行,等待被其他任务唤醒
#[inline(always)]
pub async fn suspend_now() {
    ControlFuture::new(ControlBehavior::SuspendFuture).await
}


pub struct IntrBySignalFuture {
    pub task: Arc<TaskControlBlock>,
    pub sig_mask: SigMask,
}

impl IntrBySignalFuture {
    pub fn new(task: Arc<TaskControlBlock>, sig_mask: SigMask) -> Self {
        Self { task, sig_mask }
    }
}

impl Future for IntrBySignalFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let sig_received = self.task.sig_pending.lock().has_expected(self.sig_mask).0;
        if sig_received {
            Poll::Ready(())
        }
        else {
            Poll::Pending
        }
    }
}

