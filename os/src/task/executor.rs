//! Adapted from Titanix
#![allow(unused)]

extern crate alloc;

use alloc::collections::VecDeque;
use core::future::Future;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use crate::sync::SpinNoIrqLock;


static TASK_QUEUE: TaskQueue = TaskQueue::new();

struct TaskQueue {
    normal: SpinNoIrqLock<VecDeque<Runnable>>,
    prior: SpinNoIrqLock<VecDeque<Runnable>>,
}

impl TaskQueue {
    pub const fn new() -> Self {
        Self {
            normal: SpinNoIrqLock::new(VecDeque::new()),
            prior: SpinNoIrqLock::new(VecDeque::new()),
        }
    }

    pub fn push_normal(&self, runnable: Runnable) {
        self.normal.lock().push_back(runnable);
    }

    pub fn push_prior(&self, runnable: Runnable) {
        self.prior.lock().push_back(runnable);
    }

    pub fn fetch_normal(&self) -> Option<Runnable> {
        self.normal.lock().pop_front()
    }

    pub fn fetch_prior(&self) -> Option<Runnable> {
        self.prior.lock().pop_front()
    }

    /// 取出优先级高的任务，如果没有则取出普通任务
    pub fn fetch(&self) -> Option<Runnable> {
        self.prior
            .lock()
            .pop_front()
            .or_else(|| self.normal.lock().pop_front())
    }

    pub fn len(&self) -> usize {
        self.prior_len() + self.normal_len()
    }

    pub fn prior_len(&self) -> usize {
        self.prior.lock().len()
    }

    pub fn normal_len(&self) -> usize {
        self.normal.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.prior_len() == 0 && self.normal_len() == 0
    }
}

/// 将任务加入队列
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // 在runnable.schedule()时，底层会调用这个schedule闭包，将runnable加入到任务队列中
    let schedule = move |runnable: Runnable, info: ScheduleInfo| {
        if info.woken_while_running {
            TASK_QUEUE.push_normal(runnable);
        } else {
            TASK_QUEUE.push_prior(runnable);
        }
    };
    let (runnable, task) = async_task::spawn(future, WithInfo(schedule));
    runnable.schedule();
    task.detach();
}

/// Run all tasks in the task queue
pub fn run() {
    while let Some(task) = TASK_QUEUE.fetch() {
        /// TODO:拆分TASK_QUEUE，实现steal机制
        task.run();
    }
}