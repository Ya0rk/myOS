#![allow(unused)]

extern crate alloc;

use crate::sync::{yield_now, SpinNoIrqLock, TIMER_QUEUE};
use alloc::collections::VecDeque;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use core::future::Future;

static TASK_QUEUE: TaskQueue = TaskQueue::new();

struct TaskQueue {
    idle: SpinNoIrqLock<Option<Runnable>>,
    normal: SpinNoIrqLock<VecDeque<Runnable>>,
    prior: SpinNoIrqLock<VecDeque<Runnable>>,
}

pub fn has_task() -> bool {
    TASK_QUEUE.len() > 0
}

impl TaskQueue {
    pub const fn new() -> Self {
        Self {
            idle: SpinNoIrqLock::new(None),
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
            .or_else(|| self.fetch_idle())
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

    pub fn spawn_idle(&self, idle: Runnable) {
        self.idle.lock().replace(idle);
    }

    pub fn fetch_idle(&self) -> Option<Runnable> {
        self.idle.lock().take()
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

pub fn spawn_idle<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let schedule = move |runnable: Runnable, _info: ScheduleInfo| {
        TASK_QUEUE.spawn_idle(runnable);
    };
    let (runnable, task) = async_task::spawn(future, WithInfo(schedule));
    runnable.schedule();
    task.detach();
}

/// Run all tasks in the task queue
pub fn run() {
    let mut trycnt = 0;
    loop {
        let tasks = run_once();
        if tasks == 0 {
            trycnt += 1;
        } else {
            trycnt = 0;
        }
        // 暂时注释，没有影响
        if trycnt > 0x10000000 {
            println!("no task");
            return;
        }
    }
}

pub fn run_once() -> usize {
    let mut tasks = 0;
    while let Some(task) = TASK_QUEUE.fetch() {
        task.run();
        TIMER_QUEUE.handle_expired();
        tasks += 1;
    }
    tasks
}

pub async fn yield_idle_task() {
    loop {
        yield_now().await;
    }
}
