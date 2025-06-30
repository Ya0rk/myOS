use super::{
    time::{ITimerVal, ItimerHelp},
    timer::{self, time_duration},
    SpinNoIrqLock, TimeVal,
};
use crate::{
    signal::{SigCode, SigDetails, SigErr, SigInfo, SigNom},
    sync::{once::LateInit, time::ITIMER_REAL},
    task::{get_task_by_pid, TaskControlBlock},
    utils::{Errno, SysResult},
};
use alloc::{
    collections::binary_heap::BinaryHeap,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{
    cmp::{Ordering, Reverse},
    future::Future,
    intrinsics::unlikely,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};
use log::info;
use spin::Lazy;

// TODO(YJJ):使用时间轮和最小堆混合时间管理器来优化时间复杂度===========

// 时间轮配置 - 针对600ms阈值优化
// 由于我们的内核时钟中断间隔是10ms，所以将10ms做我一个槽
const SHORT_TERM_THRESHOLD_MS: u64 = 600; // 600ms内为短期定时器
const TIME_WHEEL_SLOTS: usize = 60; // 60个槽 (600ms / 10ms)
const TIME_WHEEL_RESOLUTION_MS: u64 = 10; // 每个槽10ms (60*10=600ms)

// 定时器句柄，用于取消定时器，这里句柄的值是唯一个
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerHandle(u64);

// 定时器条目
struct TimerEntry {
    expire: Duration,     // 到期时间
    waker: Option<Waker>, // 唤醒器
    handle: TimerHandle,  // 用于取消的句柄
}

impl TimerEntry {
    pub fn new(expire: Duration, waker: Waker, handle: TimerHandle) -> Self {
        Self {
            expire,
            waker: Some(waker),
            handle,
        }
    }
}

// 时间轮结构 - 用于实现高效定时器管理的数据结构
// 时间轮算法将时间划分为多个槽(slot)，每个槽对应一个时间间隔
// 定时器根据到期时间被分配到不同的槽中，通过轮询当前槽来触发到期定时器
#[cfg(feature = "timewhell")]
struct TimingWheel {
    // 时间轮的槽数组，每个槽存储一组定时器条目
    // TIME_WHEEL_SLOTS 是时间轮的槽数量，通常为2的幂次方
    slots: [Vec<TimerEntry>; TIME_WHEEL_SLOTS],
    // 当前指向的槽索引，随着时间推移循环递增
    current_slot: usize,
    // 时间轮当前表示的时间点
    // 通常从系统启动或时间轮创建开始累计
    current_time: Duration,
}

// 定时器队列
pub struct TimerQueue {
    #[cfg(feature = "timewhell")]
    wheel: SpinNoIrqLock<TimingWheel>, // 短期定时器（<600ms）
    long_term: SpinNoIrqLock<BinaryHeap<TimerEntry>>, // 长期定时器（最小堆）
    handle_counter: SpinNoIrqLock<u64>,               // 定时器句柄计数器
}

// 全局定时器实例
lazy_static! {
    pub static ref TIMER_QUEUE: TimerQueue = TimerQueue::new();
}

// 实现时间轮
#[cfg(feature = "timewhell")]
impl TimingWheel {
    fn new(initial_time: Duration) -> Self {
        // 初始化固定大小的数组
        let slots = [(); TIME_WHEEL_SLOTS].map(|_| Vec::new());
        Self {
            slots,
            current_slot: 0,
            current_time: initial_time,
        }
    }

    fn calc_slot(&self, target_time: Duration) -> usize {
        (target_time.saturating_sub(self.current_time).as_millis() as usize)
            / TIME_WHEEL_RESOLUTION_MS as usize
    }

    // 添加定时器到时间轮
    fn add(&mut self, entry: TimerEntry) {
        let expire = entry.expire;

        // 计算槽位置（基于相对时间）
        let mut slot_offset = self.calc_slot(expire);

        // 确保在时间轮范围内
        if unlikely(slot_offset >= TIME_WHEEL_SLOTS) {
            slot_offset = TIME_WHEEL_SLOTS - 1;
        }

        let slot_index = (self.current_slot + slot_offset) % TIME_WHEEL_SLOTS;
        self.slots[slot_index].push(entry);
    }

    // 推进时间轮并返回过期定时器
    fn advance_to(&mut self, target_time: Duration) -> Vec<Waker> {
        let mut wake_list = Vec::new();

        // 计算需要推进的槽数
        let slots_to_advance = self.calc_slot(target_time);

        // 处理跨过的槽
        for _ in 0..=slots_to_advance {
            // 处理当前槽
            let slot = &mut self.slots[self.current_slot];
            let mut i = 0;
            while i < slot.len() {
                let entry = &slot[i];

                // 检查是否过期
                if entry.expire <= target_time {
                    // 添加到唤醒列表
                    wake_list.push(entry.waker.clone());
                    slot.swap_remove(i); // 移除元素（不保持顺序）
                } else {
                    i += 1; // 保留未过期元素
                }
            }

            // 移动到下一槽
            self.current_slot = (self.current_slot + 1) % TIME_WHEEL_SLOTS;
            self.current_time += Duration::from_millis(TIME_WHEEL_RESOLUTION_MS);

            // 提前退出检查
            if self.current_time > target_time {
                break;
            }
        }

        wake_list
    }
}

// 实现定时器队列
impl TimerQueue {
    pub fn new() -> Self {
        let initial_time = time_duration();
        Self {
            #[cfg(feature = "timewhell")]
            wheel: SpinNoIrqLock::new(TimingWheel::new(initial_time)),
            long_term: SpinNoIrqLock::new(BinaryHeap::new()),
            handle_counter: SpinNoIrqLock::new(0),
        }
    }

    /// 自动+1生成新的handle,生成唯一个timer的id号
    pub fn new_handle(&self) -> TimerHandle {
        let mut handle_counter = self.handle_counter.lock();
        *handle_counter += 1;
        TimerHandle(*handle_counter)
    }

    // 添加定时器
    pub fn add_timer(&self, timer: TimerEntry) {
        #[cfg(feature = "timewhell")]
        {
            let expire = timer.expire;
            let current_time = time_duration().as_millis() as u64;

            // 根据阈值决定放入时间轮还是堆
            if expire.as_millis() as u64 <= current_time + SHORT_TERM_THRESHOLD_MS {
                // 短期：放入时间轮
                self.wheel.lock().add(timer);
                return;
            }
        }

        // 长期：放入堆
        self.long_term.lock().push(timer);
    }

    // 取消定时器
    #[cfg(feature = "timewhell")]
    pub fn cancel(&self, handle: TimerHandle) {
        // 尝试从时间轮取消
        let mut wheel = self.wheel.lock();
        for slot in &mut wheel.slots {
            let mut i = 0;
            while i < slot.len() {
                if slot[i].handle == handle {
                    slot.swap_remove(i); // 直接移除
                    return;
                } else {
                    i += 1;
                }
            }
        }
        drop(wheel);

        // 尝试从堆中取消
        let mut long_term = self.long_term.lock();
        let mut temp = Vec::new();

        while let Some(entry) = long_term.pop() {
            if entry.handle == handle {
                // 找到并丢弃
                return;
            }
            temp.push(entry);
        }

        // 将未取消的条目放回堆中
        for entry in temp {
            long_term.push(entry);
        }
    }

    // 处理过期定时器（应在系统时钟中断中调用）
    pub fn handle_expired(&self) {
        let mut wake_list = Vec::new();
        let current_time = time_duration();

        // 处理时间轮（获取过期定时器）
        #[cfg(feature = "timewhell")]
        wake_list.extend(self.wheel.lock().advance_to(current_time));

        // 处理长期定时器
        {
            let mut long_term = self.long_term.lock();

            while let Some(entry) = long_term.peek() {
                if entry.expire >= current_time {
                    break;
                }
                // 过期：直接唤醒
                if let Some(entry) = long_term.pop() {
                    wake_list.extend(entry.waker);
                }
            }
        }

        // 唤醒所有任务（在锁外执行）
        for waker in wake_list {
            waker.wake();
        }
    }
}

/// 超时Future，会在deadline之前反复调用，直到执行完成
/// 如果超时后没有完成，就返回超时；和下面的定时任务不一样
pub struct TimeoutFuture<F: Future> {
    inner: F,
    deadline: Duration,
    timer_handle: Option<TimerHandle>,
}

impl<F: Future> TimeoutFuture<F> {
    pub fn new(inner: F, span: Duration) -> Self {
        Self {
            inner,
            deadline: time_duration() + span,
            timer_handle: None,
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

        // 注册/更新定时器
        if this.timer_handle.is_none() {
            let handle = TIMER_QUEUE.new_handle();
            let timer = TimerEntry::new(this.deadline, cx.waker().clone(), handle);
            TIMER_QUEUE.add_timer(timer);
            this.timer_handle = Some(handle);
        }

        Poll::Pending
    }
}

#[cfg(feature = "timewhell")]
impl<F: Future> Drop for TimeoutFuture<F> {
    fn drop(&mut self) {
        // 确保定时器被取消
        if let Some(handle) = self.timer_handle.take() {
            TIMER_QUEUE.cancel(handle);
        }
    }
}

// 实现排序
impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expire.cmp(&other.expire).reverse()
    }
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.expire == other.expire
    }
}

impl Eq for TimerEntry {}

// ======================================
// pub struct TimerQueue {
//     timers: SpinNoIrqLock<BinaryHeap<TimerTranc>>, // 直接使用最小堆
// }

// impl TimerQueue {
//     pub fn new() -> Self {
//         Self {
//             timers: SpinNoIrqLock::new(BinaryHeap::new()),
//         }
//     }

//     /// 添加定时器（O(log n)）
//     pub fn add(&self, timer: TimerTranc) {
//         let mut heap = self.timers.lock();
//         heap.push(timer);
//     }

//     /// 处理过期事件（O(k log n) k为过期事件数）
//     pub fn handle_expired(&self) {
//         let mut wake_list = Vec::new();
//         let current_ns = time_duration();

//         {
//             let mut heap = self.timers.lock();
//             while let Some(timer) = heap.peek() {
//                 if timer.expire_ns >= current_ns {
//                     break;
//                 }
//                 if let Some(timer) = heap.pop() {
//                     wake_list.extend(timer.waker);
//                 }
//             }
//         } // 提前释放锁

//         for waker in wake_list {
//             info!("[TimerQueue] wake up task");
//             waker.wake(); // 在锁外执行唤醒
//         }
//     }
// }

// pub static TIMER_QUEUE: Lazy<TimerQueue> = Lazy::new(|| TimerQueue::new());

// /// 超时Future，会在deadline之前反复调用，直到执行完成
// /// 如果超时后没有完成，就返回超时；和下面的定时任务不一样
// pub struct TimeoutFuture<F: Future> {
//     inner: F,
//     deadline: Duration,
//     timer_registered: bool,
// }

// impl<F: Future> TimeoutFuture<F> {
//     pub fn new(inner: F, span: Duration) -> Self {
//         Self {
//             inner,
//             deadline: time_duration() + span,
//             timer_registered: false,
//         }
//     }
// }

// impl<F: Future> Future for TimeoutFuture<F> {
//     type Output = Result<F::Output, Errno>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         // 先检查内部Future是否完成
//         let this = unsafe { self.get_unchecked_mut() };
//         let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
//         // println!(
//         //     "timeout future start: polling inner future, deadline = {:?}",
//         //     this.deadline
//         // );
//         if let Poll::Ready(v) = inner.poll(cx) {
//             return Poll::Ready(Ok(v));
//         }

//         // 检查是否已经超时
//         let current = time_duration();
//         // println!(
//         //     "timeout future: checking time, current = {:?}, deadline = {:?}",
//         //     current, this.deadline
//         // );
//         if current >= this.deadline {
//             info!("[TimeoutFuture] time use out");
//             return Poll::Ready(Err(Errno::ETIMEDOUT));
//         }

//         // 注册定时器（仅一次）
//         if !this.timer_registered {
//             let deadline = this.deadline;
//             TIMER_QUEUE.add(TimerTranc::new(deadline, cx.waker().clone()));
//             this.timer_registered = true;
//         }

//         Poll::Pending
//     }
// }

/// 用来空转,如果被信号kill、stop打断，那么返回剩余时间
pub struct NullFuture {
    pub task: Arc<TaskControlBlock>,
    pub deadline: Duration, // 空转的时间
}

impl NullFuture {
    pub fn new(task: Arc<TaskControlBlock>, deadline: Duration) -> Self {
        Self { task, deadline }
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
    pub fn new(
        next_expire: Duration,
        callback: F,
        task: Arc<TaskControlBlock>,
        which: usize,
    ) -> Self {
        Self {
            next_expire,
            task: Arc::downgrade(&task),
            callback,
            which,
        }
    }
}

impl<F: Fn() -> bool> Future for ItimerFuture<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let cur_time = time_duration();

        if cur_time >= this.next_expire {
            if !((this.callback)()) {
                // 从闭包下面的中拿出来，避免死锁
                return Poll::Ready(());
            }
        }
        let res = this
            .task
            .upgrade()
            .and_then(|task| {
                // 加入时钟队列
                let tmp = task.whit_itimers(|itimers| {
                    let real_time = itimers[this.which];
                    this.next_expire = (Duration::from(real_time.it_interval) + time_duration())
                        .max(this.next_expire);
                    // let new_timer = TimerTranc::new(this.next_expire, cx.waker().clone());
                    let new_timer = TimerEntry::new(
                        this.next_expire,
                        cx.waker().clone(),
                        TIMER_QUEUE.new_handle(),
                    );
                    // TIMER_QUEUE.add(new_timer);
                    TIMER_QUEUE.add_timer(new_timer);
                    Poll::Pending
                });
                Some(tmp)
            })
            .unwrap();
        res
    }
}

pub fn itimer_callback(pid: usize, iterval: TimeVal, which: usize) -> bool {
    let task = match get_task_by_pid(pid) {
        Some(task) => task,
        _ => return false,
    };

    if task.itimers.lock()[which].it_value.is_zero() {
        return false;
    }
    task.itimers.lock()[which].it_value = (Duration::from(iterval) + time_duration()).into();

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
