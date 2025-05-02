use core::{future::Future, pin::Pin, task::{Context, Poll}, time::Duration};

use alloc::{collections::binary_heap::BinaryHeap, vec::Vec};
use log::info;
use spin::Lazy;
use crate::utils::Errno;

use super::{timer::{time_duration, Timer}, SpinNoIrqLock};

// TODO(YJJ):使用时间轮和最小堆混合时间管理器来优化时间复杂度===========

pub struct TimerQueue {
    timers: SpinNoIrqLock<BinaryHeap<Timer>>, // 直接使用最小堆
}

impl TimerQueue {
    pub fn new() -> Self {
        Self {
            timers: SpinNoIrqLock::new(BinaryHeap::new()),
        }
    }

    /// 添加定时器（O(log n)）
    pub fn add(&self, timer: Timer) {
        let mut heap = self.timers.lock();
        heap.push(timer);
    }

    /// 处理过期事件（O(k log n) k为过期事件数）
    pub fn handle_expired(&self) {
        let mut wake_list = Vec::new();
        let current_ns = time_duration().as_nanos() as u64;
        
        {
            let mut heap = self.timers.lock();
            while let Some(timer) = heap.peek() {
                if timer.expire_ns > current_ns {
                    break;
                }
                if let Some(timer) = heap.pop() {
                    wake_list.extend(timer.waker);
                }
            }
        } // 提前释放锁
        
        for waker in wake_list {
            waker.wake(); // 在锁外执行唤醒
        }
    }
}

pub static TIMER_QUEUE: Lazy<TimerQueue> = Lazy::new(|| TimerQueue::new());

/// 超时Future
pub struct TimeoutFuture<F: Future> {
    inner: F,
    deadline: Duration,
    timer_registered: bool,
}

impl<F: Future> TimeoutFuture<F> {
    pub fn new(inner: F, timeout: Duration) -> Self {
        Self {
            inner,
            deadline: time_duration() + timeout,
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
        if current >= this.deadline {
            info!("[TimeoutFuture] time use out");
            return Poll::Ready(Err(Errno::ETIMEDOUT));
        }

        // 注册定时器（仅一次）
        if !this.timer_registered {
            let deadline = this.deadline;
            TIMER_QUEUE.add(Timer::new(deadline, cx.waker().clone()));
            this.timer_registered = true;
        }

        Poll::Pending
    }
}