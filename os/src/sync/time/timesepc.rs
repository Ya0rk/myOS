use core::time::Duration;
use zerocopy::IntoBytes;
use crate::{sync::timer::{get_time_ns, get_time_s, NSEC_PER_SEC}, utils::Errno};

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

        Ok(TimeSepc { tv_sec, tv_nsec })
    }

    pub fn check_valid(&self) -> bool {
        (self.tv_nsec < NSEC_PER_SEC) && (self.tv_sec > 0) && (self.tv_nsec > 0)
    }
}

impl From<TimeSepc> for Duration {
    fn from(ts: TimeSepc) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}