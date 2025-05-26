use core::{cmp::Ordering, task::Waker, time::Duration};
use crate::{hal::arch::set_timer, hal::config::CLOCK_FREQ};
use super::{time::TimeSpec, yield_now, SpinNoIrqLock};
use alloc::collections::binary_heap::BinaryHeap;
use spin::Lazy;

const TICKS_PER_SEC: usize = 100; // 设置每秒中断次数，可以计算出每次中断的时间间隔
pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;
pub const NSEC_PER_SEC: usize = 1_000_000_000;
pub const TIME_SLICE_DUATION: Duration = Duration::new(0, (NSEC_PER_SEC / TICKS_PER_SEC) as u32);

use crate::hal::arch::get_time;

/// 获取时间：单位s
#[inline(always)]
pub fn get_time_s() -> usize {
    get_time() / CLOCK_FREQ
}

/// 获取时间：单位ms
#[inline(always)]
/// get current time in microseconds
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MSEC_PER_SEC)
}

/// 获取时间：单位us
#[inline(always)]
pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / USEC_PER_SEC)
}

/// 获取时间：单位ns
#[inline(always)]
pub fn get_time_ns() -> usize {
    (get_time() / (CLOCK_FREQ / USEC_PER_SEC)) * MSEC_PER_SEC
}

pub fn time_duration() -> Duration {
    Duration::from_micros(get_time_us() as u64)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub async fn sleep_for(ts: TimeSpec) {
    let start = get_time_ms();
    let span = ts.tv_sec * 1_000 + ts.tv_nsec / 1_000_000;
    while get_time_ms() - start < span { // 这之前单位有点问题
        yield_now().await;
    }
}

#[derive(Debug)]
pub struct TimerTranc {
    pub expire_ns: Duration,  // 使用纳秒精度
    pub waker: Option<Waker>, // 非空保证
}

impl TimerTranc {
    pub fn new(expire: Duration, waker: Waker) -> Self {
        Self {
            expire_ns: expire,
            waker: Some(waker),
        }
    }
}

// 实现按过期时间排序（最小堆）
impl Ord for TimerTranc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expire_ns.cmp(&other.expire_ns).reverse() // 反向实现最小堆
        // self.expire_ns.cmp(&other.expire_ns)
    }
}

impl PartialOrd for TimerTranc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TimerTranc {}

impl PartialEq for TimerTranc {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ns == other.expire_ns
    }
}