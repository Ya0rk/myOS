#![allow(unused)]

extern crate alloc;

use alloc::{collections::VecDeque, task};
use log::info;
use core::{future::Future, sync::atomic::{AtomicU32, AtomicUsize}};
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use crate::{config::HART_NUM, sync::SpinNoIrqLock};

use super::get_current_hart_id;

const QUEUE_NUM: usize = HART_NUM;
const INDEX: AtomicUsize = AtomicUsize::new(0);
static TASK_QUEUES: [TaskQueue; QUEUE_NUM] = [
    TaskQueue::new(),
    TaskQueue::new(),
];

struct TaskQueue {
    normal: SpinNoIrqLock<VecDeque<Runnable>>,
    prior: SpinNoIrqLock<VecDeque<Runnable>>,
}

impl TaskQueue {
    const fn new() -> Self {
        Self {
            normal: SpinNoIrqLock::new(VecDeque::new()),
            prior: SpinNoIrqLock::new(VecDeque::new()),
        }
    }

    fn push_n_normal(&self, mut runnables: VecDeque<Runnable>) {
        self.normal.lock().append(&mut runnables);
    }

    fn push_normal(&self, runnable: Runnable) {
        self.normal.lock().push_back(runnable);
    }

    fn push_prior(&self, runnable: Runnable) {
        self.prior.lock().push_back(runnable);
    }

    /// 取出优先级高的任务，如果没有则取出普通任务
    fn fetch(&self) -> Option<Runnable> {
        self.prior
            .lock()
            .pop_front()
            .or_else(|| self.normal.lock().pop_front())
    }

    /// 从任务队列窃取一半的normal task，可以减少窃取次数
    fn steal(&self) -> Option<VecDeque<Runnable>> {
        info!("steal task---");
        let len = self.normal_len();
        if len == 0 {
            return None;
        }
        let mut normal_lock = self.normal.lock();
        let steal_cont = len / 2;
        let stealen_tasks = normal_lock.split_off(steal_cont);
        Some(stealen_tasks)
    }

    fn len(&self) -> usize {
        self.prior_len() + self.normal_len()
    }

    fn prior_len(&self) -> usize {
        self.prior.lock().len()
    }

    fn normal_len(&self) -> usize {
        self.normal.lock().len()
    }

    fn is_empty(&self) -> bool {
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
        let queue_idx = get_current_hart_id();
        let queue = &TASK_QUEUES[queue_idx];
        if info.woken_while_running {
            queue.push_normal(runnable);
        } else {
            queue.push_prior(runnable);
        }
    };
    let (runnable, task) = async_task::spawn(future, WithInfo(schedule));
    runnable.schedule();
    task.detach();
}

pub fn tasks_len() -> usize {
    let mut len = 0;
    for i in 0..QUEUE_NUM {
        len += TASK_QUEUES[i].len()
    }
    len
}

pub fn has_task() -> bool {
    tasks_len() > 0
}

/// Run all tasks in the task queue
pub fn run() {
    let mut steal_counter = 0;
    loop {
        // 首先尝试从自己的队列中获取任务
        let worker_id = get_current_hart_id();
        if let Some(task) = TASK_QUEUES[worker_id].fetch() {
            // info!("nihao");
            task.run();
            // info!("sssss");
            steal_counter = 0; // 重置窃取计数器
        } else {
            // 如果自己的队列为空，尝试从其他队列中窃取任务
            // info!("bhao");
            steal_counter += 1;
            if steal_counter > QUEUE_NUM * 2 {
                // 如果多次窃取失败，可能没有任务了，退出循环
                break;
            }
            for i in 0..QUEUE_NUM {
                if i == worker_id {
                    continue; // 跳过自己的队列
                }
                if let Some(tasks) = TASK_QUEUES[i].steal() {
                    TASK_QUEUES[worker_id].push_n_normal(tasks); // 将偷取的任务加入自己队列
                    steal_counter = 0; // 重置窃取计数器
                    break;
                }
            }
        }
    }
}

// use core::future::Future;

// use alloc::collections::vec_deque::VecDeque;
// use async_task::{Runnable, ScheduleInfo, WithInfo};

// use crate::sync::SpinNoIrqLock;

// lazy_static! {
//     static ref EXECUTOR: Executor = Executor::new();
// }
// struct Executor {
//     task_queue: SpinNoIrqLock<VecDeque<Runnable>>,
// }

// impl Executor {
//     pub fn new() -> Self {
//         Self {
//             task_queue: SpinNoIrqLock::new(VecDeque::new()),
//         }
//     }
//     pub fn push_back(&self, runnable: Runnable) {
//         self.task_queue.lock().push_back(runnable);
//     }

//     pub fn push_front(&self, runnable: Runnable) {
//         self.task_queue.lock().push_front(runnable);
//     }

//     pub fn fetch(&self) -> Option<Runnable> {
//         self.task_queue.lock().pop_front()
//     }
// }

// /// Add a task into task queue
// pub fn spawn<F, R>(future: F)
// where
//     F: Future<Output = R> + Send + 'static,
//     R: Send + 'static,
// {
//     let schedule = move |task: Runnable, info: ScheduleInfo| {
//         if info.woken_while_running {
//             EXECUTOR.push_back(task);
//         } else {
//             EXECUTOR.push_front(task);
//         }
//     };
//     // let schedule = |task| EXECUTOR.push_back(task);
//     let (task, handle) = async_task::spawn(future, WithInfo(schedule));
//     task.schedule();
//     handle.detach();
// }

// pub fn run() {
//     loop {
//         if let Some(task) = EXECUTOR.fetch() {
//             task.run();
//             // handle_timeout_events();
//         } else {
//             break;
//         }
//     }
// }

// pub fn has_task() -> bool {
//     EXECUTOR.task_queue.lock().len() >= 1
// }