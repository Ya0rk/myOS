use crate::{arch::set_timer, config::CLOCK_FREQ, task::suspend_current_and_run_next};
use riscv::register::time;
use super::time::TimeSepc;

const TICKS_PER_SEC: usize = 100;
pub const MSEC_PER_SEC: usize = 1000;

#[inline(always)]
///get current time
pub fn get_time() -> usize {
    time::read()
}

#[inline(always)]
/// get current time in microseconds
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

#[inline(always)]
pub fn get_time_s() -> usize {
    get_time_ms() / MSEC_PER_SEC
}

#[inline(always)]
pub fn get_time_us() -> usize {
    (get_time_ms() % MSEC_PER_SEC) * MSEC_PER_SEC
}

#[inline(always)]
pub fn get_time_ns() -> usize {
    (get_time_ms() % MSEC_PER_SEC) * MSEC_PER_SEC * MSEC_PER_SEC
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