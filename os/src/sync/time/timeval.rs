use core::time::Duration;

use zerocopy::{Immutable, IntoBytes};
use crate::{board::CLOCK_FREQ, hal::arch::get_time, sync::timer::{get_time_s, get_time_us, USEC_PER_SEC}};

#[derive(Clone, Copy, IntoBytes, Immutable, Default)]
#[repr(C)]
pub struct TimeVal {
    /// 秒
    pub tv_sec: usize,
    /// 微秒
    pub tv_usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        let tick = get_time_us();
        TimeVal { tv_sec: tick / USEC_PER_SEC, tv_usec: tick % USEC_PER_SEC }
    }
}

impl From<Duration> for TimeVal {
    fn from(value: Duration) -> Self {
        Self {
            tv_sec: value.as_secs() as usize,
            tv_usec: value.subsec_micros() as usize
        }
    }
}