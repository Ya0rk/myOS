// #![allow(unused)]
// use alloc::{collections::VecDeque, task};
// use log::info;
// use core::{future::Future, sync::atomic::{AtomicU32, AtomicUsize}};
// use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
// use crate::{hal::config::HART_NUM, sync::{SpinNoIrqLock, TIMER_QUEUE}, utils::{LcgRng, RNG}};
// use super::get_current_hart_id;

// const QUEUE_NUM: usize = HART_NUM;
// const INDEX: AtomicUsize = AtomicUsize::new(0);
// static TASK_QUEUES: [TaskQueue; QUEUE_NUM] = [
//     TaskQueue::new(),
//     TaskQueue::new(),
// ];

// struct TaskQueue {
//     normal: SpinNoIrqLock<VecDeque<Runnable>>,
//     prior: SpinNoIrqLock<VecDeque<Runnable>>,
// }

// impl TaskQueue {
//     const fn new() -> Self {
//         Self {
//             normal: SpinNoIrqLock::new(VecDeque::new()),
//             prior: SpinNoIrqLock::new(VecDeque::new()),
//         }
//     }

//     fn push_n_normal(&self, mut runnables: VecDeque<Runnable>) {
//         self.normal.lock().append(&mut runnables);
//     }

//     fn push_normal(&self, runnable: Runnable) {
//         self.normal.lock().push_back(runnable);
//     }

//     fn push_prior(&self, runnable: Runnable) {
//         self.prior.lock().push_back(runnable);
//     }

//     /// 取出优先级高的任务，如果没有则取出普通任务
//     fn fetch(&self) -> Option<Runnable> {
//         self.prior
//             .lock()
//             .pop_front()
//             .or_else(|| self.normal.lock().pop_front())
//     }

//     /// 从任务队列窃取一半的normal task，可以减少窃取次数
//     fn steal(&self) -> Option<VecDeque<Runnable>> {
//         // info!("steal task---");
//         let len = self.normal_len();
//         if len == 0 {
//             return None;
//         }
//         let mut normal_lock = self.normal.lock();
//         let steal_cont = len / 2;
//         let stealen_tasks = normal_lock.split_off(steal_cont);
//         Some(stealen_tasks)
//     }

//     fn len(&self) -> usize {
//         self.prior_len() + self.normal_len()
//     }

//     fn prior_len(&self) -> usize {
//         self.prior.lock().len()
//     }

//     fn normal_len(&self) -> usize {
//         self.normal.lock().len()
//     }

//     fn is_empty(&self) -> bool {
//         self.prior_len() == 0 && self.normal_len() == 0
//     }
// }

// /// 将任务加入队列
// pub fn spawn<F>(future: F)
// where
//     F: Future + Send + 'static,
//     F::Output: Send + 'static,
// {
//     // 在runnable.schedule()时，底层会调用这个schedule闭包，将runnable加入到任务队列中
//     let schedule = move |runnable: Runnable, info: ScheduleInfo| {
//         let queue_idx = get_current_hart_id();
//         let queue = &TASK_QUEUES[queue_idx];
//         if info.woken_while_running {
//             queue.push_normal(runnable);
//         } else {
//             queue.push_prior(runnable);
//         }
//     };
//     let (runnable, task) = async_task::spawn(future, WithInfo(schedule));
//     runnable.schedule();
//     task.detach();
// }

// pub fn tasks_len() -> usize {
//     let mut len = 0;
//     for i in 0..QUEUE_NUM {
//         len += TASK_QUEUES[i].len()
//     }
//     len
// }

// pub fn has_task() -> bool {
//     tasks_len() > 0
// }

// /// Run all tasks in the task queue
// pub fn run() {
//     let mut steal_counter = 0;
//     let mut trycnt = 0;
//     loop {
//         let tasks = run_once();
//         if tasks == 0 {
//             trycnt += 1;
//             steal_counter += 1;
//             // if steal_counter > QUEUE_NUM * 2 {
//             //     // 如果多次窃取失败，可能没有任务了，退出循环
//             //     break;
//             // }
//             // for i in 0..QUEUE_NUM {
//             //     if i == worker_id {
//             //         continue; // 跳过自己的队列
//             //     }
//             //     if let Some(tasks) = TASK_QUEUES[i].steal() {
//             //         TASK_QUEUES[worker_id].push_n_normal(tasks); // 将偷取的任务加入自己队列
//             //         steal_counter = 0; // 重置窃取计数器
//             //         break;
//             //     }
//             // }

//         } else {
//             trycnt = 0;
//             steal_counter = 0; // 重置窃取计数器
//         }
//         if trycnt > 0x10000000 {
//             println!("no task");
//             return;
//         }
//     }
// }

// pub fn run_once() -> usize {
//     let mut tasks = 0;
//     let mut steal_cnt = 0;
//     let worker_id = get_current_hart_id();
//     if let Some(task) = TASK_QUEUES[worker_id].fetch() {
//         task.run();
//         tasks += 1;
//     } else {
//         steal_cnt += 1;
//         if steal_cnt > QUEUE_NUM * 10 {
//             // 如果多次窃取失败，可能没有任务了，退出循环
//             println!("no task");
//             return 0;
//         }
//     }
//     tasks
// }

#![allow(unused)]

extern crate alloc;

use crate::sync::SpinNoIrqLock;
use alloc::collections::VecDeque;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use core::future::Future;

static TASK_QUEUE: TaskQueue = TaskQueue::new();

struct TaskQueue {
    normal: SpinNoIrqLock<VecDeque<Runnable>>,
    prior: SpinNoIrqLock<VecDeque<Runnable>>,
}

pub fn has_task() -> bool {
    TASK_QUEUE.len() > 0
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
    let mut trycnt = 0;
    loop {
        let tasks = run_once();
        if tasks == 0 {
            trycnt += 1;
        } else {
            trycnt = 0;
        }
        if trycnt > 0x10000000 {
            println!("no task");
            return;
        }
    }
    // while let Some(task) = TASK_QUEUE.fetch() {
    //     task.run();
    // }
}

pub fn run_once() -> usize {
    let mut tasks = 0;
    while let Some(task) = TASK_QUEUE.fetch() {
        task.run();
        tasks += 1;
    }
    tasks
}
