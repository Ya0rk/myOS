use core::{ops::Sub, time::Duration};
use zerocopy::{Immutable, IntoBytes};
use crate::{board::CLOCK_FREQ, hal::arch::get_time, sync::{time_duration, timer::{get_time_s, get_time_us, USEC_PER_SEC}}};

#[derive(Clone, Copy, IntoBytes, Immutable, Default)]
#[repr(C)]
pub struct TimeVal {
    /// 秒
    pub tv_sec: usize,
    /// 微秒
    pub tv_usec: usize,
}

impl TimeVal {
    /// 以当前时间创建一个新的 `TimeVal` 实例
    pub fn new() -> Self {
        let tick = get_time_us();
        TimeVal { tv_sec: tick / USEC_PER_SEC, tv_usec: tick % USEC_PER_SEC }
    }

    pub fn is_zero(&self) -> bool {
        self.tv_sec == 0 && self.tv_usec == 0
    }

    pub fn set(&mut self, sec: usize, usec: usize) {
        self.tv_sec = sec;
        self.tv_usec = usec;
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

impl From<TimeVal> for Duration {
    fn from(value: TimeVal) -> Self {
        Duration::new(value.tv_sec as u64, (value.tv_usec * 1000) as u32)
    }
}

impl Sub for TimeVal {
    type Output = Self;

    /// 这里实现的是时间的差值，结果最小为0，不能是负数
    fn sub(self, other: Self) -> Self {
        let sself = Duration::from(self);
        let sother = Duration::from(other);
        let diff = sself.checked_sub(sother).unwrap_or(Duration::ZERO);
        TimeVal::from(diff)
    }
}