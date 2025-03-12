use core::time::Duration;
use crate::{arch::set_timer, config::CLOCK_FREQ, task::suspend_current_and_run_next};
use riscv::register::time;
use super::time::TimeSepc;

const TICKS_PER_SEC: usize = 100; // 设置每秒中断次数，可以计算出每次中断的时间间隔
pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;
pub const NSEC_PER_SEC: usize = 1_000_000_000;

#[inline(always)]
/// 获取开机以来，晶振片过了几个时钟周期
pub fn get_time() -> usize {
    time::read()
}

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
    get_time() / (CLOCK_FREQ / NSEC_PER_SEC)
}

pub fn time_duration() -> Duration {
    Duration::from_nanos(get_time_us() as u64)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn sleep_for(ts: TimeSepc) {
    let start = get_time_ms();
    let span = ts.tv_sec * 1_000 + ts.tv_nsec / 1_000_000;
    while get_time_ns() - start < span {
        suspend_current_and_run_next();
    }
}