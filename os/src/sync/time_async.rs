use core::{cmp::Reverse, future::Future, pin::Pin, task::{Context, Poll}, time::Duration};
use alloc::{collections::binary_heap::BinaryHeap, sync::{Arc, Weak}, vec::Vec};
use log::info;
use spin::Lazy;
use crate::{signal::{SigCode, SigDetails, SigErr, SigInfo, SigNom}, task::{get_task_by_pid, TaskControlBlock}, utils::{Errno, SysResult}};
use super::{time::{ITimerVal, ItimerHelp}, timer::{self, time_duration, TimerTranc}, SpinNoIrqLock, TimeVal};

// TODO(YJJ):使用时间轮和最小堆混合时间管理器来优化时间复杂度===========

pub struct TimerQueue {
    timers: SpinNoIrqLock<BinaryHeap<TimerTranc>>, // 直接使用最小堆
}

impl TimerQueue {
    pub fn new() -> Self {
        Self {
            timers: SpinNoIrqLock::new(BinaryHeap::new()),
        }
    }

    /// 添加定时器（O(log n)）
    pub fn add(&self, timer: TimerTranc) {
        let mut heap = self.timers.lock();
        heap.push(timer);
    }

    /// 处理过期事件（O(k log n) k为过期事件数）
    pub fn handle_expired(&self) {
        let mut wake_list = Vec::new();
        let current_ns = time_duration();
        
        {
            let mut heap = self.timers.lock();
            while let Some(timer) = heap.peek() {
                if timer.expire_ns >= current_ns {
                    break;
                }
                if let Some(timer) = heap.pop() {
                    wake_list.extend(timer.waker);
                }
            }
        } // 提前释放锁
        
        for waker in wake_list {
            info!("[TimerQueue] wake up task");
            waker.wake(); // 在锁外执行唤醒
        }
    }
}

pub static TIMER_QUEUE: Lazy<TimerQueue> = Lazy::new(|| TimerQueue::new());

/// 超时Future，会在deadline之前反复调用，直到执行完成
/// 如果超时后没有完成，就返回超时；和下面的定时任务不一样
pub struct TimeoutFuture<F: Future> {
    inner: F,
    deadline: Duration,
    timer_registered: bool,
}

impl<F: Future> TimeoutFuture<F> {
    pub fn new(inner: F, span: Duration) -> Self {
        Self {
            inner,
            deadline: time_duration() + span,
            timer_registered: false,
        }
    }
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, Errno>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 先检查内部Future是否完成
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        if let Poll::Ready(v) = inner.poll(cx) {
            return Poll::Ready(Ok(v));
        }

        // 检查是否已经超时
        let current = time_duration();
        info!("timeout future: checking time, current = {:?}, deadline = {:?}", current, this.deadline);
        if current >= this.deadline {
            info!("[TimeoutFuture] time use out");
            return Poll::Ready(Err(Errno::ETIMEDOUT));
        }

        // 注册定时器（仅一次）
        if !this.timer_registered {
            let deadline = this.deadline;
            TIMER_QUEUE.add(TimerTranc::new(deadline, cx.waker().clone()));
            this.timer_registered = true;
        }

        Poll::Pending
    }
}

/// 用来空转,如果被信号kill、stop打断，那么返回剩余时间
pub struct NullFuture {
    pub task: Arc<TaskControlBlock>,
    pub deadline: Duration // 空转的时间
}

impl NullFuture {
    pub fn new(task: Arc<TaskControlBlock>, deadline: Duration) -> Self {
        Self {
            task,
            deadline
        }
    }
}

impl Future for NullFuture {
    type Output = SysResult<Duration>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if this.task.rv_intr() {
            let cur = time_duration();
            let left = this.deadline - cur;
            info!("[idelfuture] be interupt");
            return Poll::Ready(Ok(left));
        }
        Poll::Pending
    }
}

/// 用于itimer系统调用设置定时任务
/// 超过时间就调用callback一次
pub struct ItimerFuture<F: Fn() -> bool> {
    pub next_expire: Duration,
    pub task: Weak<TaskControlBlock>,
    pub callback: F,
    pub which: usize,
}

impl<F: Fn() -> bool> ItimerFuture<F> {
    pub fn new(next_expire: Duration, callback: F, task: Arc<TaskControlBlock>, which: usize) -> Self {
        Self {
            next_expire,
            task: Arc::downgrade(&task),
            callback,
            which
        }
    }

}

impl<F: Fn() -> bool> Future for ItimerFuture<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe{ self.get_unchecked_mut() };
        let res = this.task.upgrade().and_then(|task|{
            // 加入时钟队列
            let tmp = task.whit_itimers(|itimers| {
                let cur_time = time_duration();
                let real_time = itimers[this.which];

                if cur_time >= this.next_expire {
                    if !((this.callback)()) {
                        return Some(Poll::Ready(()));
                    }
                }
                let new_timer = TimerTranc::new(this.next_expire, cx.waker().clone());
                TIMER_QUEUE.add(new_timer);
                this.next_expire = (Duration::from(real_time.it_interval) + time_duration());
                Some(Poll::Pending)
            });
            tmp
        }).unwrap();
        res
    }
}

pub fn itimer_callback(pid: usize, iterval: TimeVal) -> bool {
    let task = match get_task_by_pid(pid) {
        Some(task) => task,
        _ => return false,
    };

    let siginfo = SigInfo {
        signo: SigNom::SIGALRM,
        sigcode: SigCode::Kernel,
        sigerr: SigErr::empty(),
        sifields: SigDetails::None,
    };

    task.proc_recv_siginfo(siginfo);

    // 如果iterval为0，代表不触发下一次
    if iterval.is_zero() {
        return false;
    }

    return true;
}