use crate::{hal::arch::set_timer, config::CLOCK_FREQ, task::suspend_current_and_run_next, utils::Errno};
// use riscv::register::time;
use zerocopy::{IntoBytes, Immutable};
use core::time::Duration;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub struct TimeVal {
    /// 秒
    pub tv_sec: usize,
    /// 微秒
    pub tv_usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        TimeVal { tv_sec: get_time_s(), tv_usec: get_time_us() }
    }
}

#[derive(Copy, Clone, IntoBytes)]
#[repr(C)]
pub struct TimeSepc {
    /// 秒
    pub tv_sec: usize,
    /// 纳秒
    pub tv_nsec: usize,
}

impl TimeSepc {
    pub fn new() -> Result<Self, Errno> {
        let tv_sec = get_time_s();
        let tv_nsec = get_time_ns();

        if tv_nsec >= MSEC_PER_SEC * MSEC_PER_SEC * MSEC_PER_SEC {
            return Err(Errno::EINVAL);
        }

        Ok(TimeSepc { tv_sec, tv_nsec })
    }

    pub fn check_valid(&self) -> bool {
        self.tv_nsec < MSEC_PER_SEC * MSEC_PER_SEC * MSEC_PER_SEC
    }
}

impl From<TimeSepc> for Duration {
    fn from(ts: TimeSepc) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

#[allow(non_camel_case_types)]
type clock_t = isize;

#[derive(IntoBytes, Immutable)]
pub struct Tms {
    /// 进程在用户态（user mode）消耗的 CPU 时间
    pub tms_utime: clock_t,
    /// 进程在内核态（system mode）消耗的 CPU 时间
    pub tms_stime: clock_t,
    /// 所有已终止子进程的用户态时间总和： sum of the tms_utime and tms_cutime
    pub tms_cutime: clock_t,
    /// 所有已终止子进程的内核态时间总和： sum of the tms_stime and tms_cstime
    pub tms_cstime: clock_t,
}

impl Tms {
    pub fn new() -> Self {
        let now = get_time_ms() as clock_t;
        Self {
            tms_utime: now,
            tms_stime: now,
            tms_cutime: now,
            tms_cstime: now,
        }
    }
}

#[inline(always)]
///get current time
pub fn get_time() -> usize {
    // time::read()
    unimplemented!()
}

#[inline(always)]
/// get current time in microseconds
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MSEC_PER_SEC)
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