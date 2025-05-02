use core::{fmt::Display, time::Duration};
use zerocopy::IntoBytes;
use crate::{sync::timer::{get_time_ns, get_time_s, time_duration, NSEC_PER_SEC}, task::current_task};

#[derive(Copy, Clone, IntoBytes)]
#[repr(C)]
pub struct TimeSpec {
    /// 秒
    pub tv_sec: usize,
    /// 纳秒
    pub tv_nsec: usize,
}

impl TimeSpec {
    pub fn new() -> Self {
        let tv_sec = get_time_s();
        let tv_nsec = get_time_ns();

        TimeSpec { tv_sec, tv_nsec }
    }

    pub fn check_valid(&self) -> bool {
        self.tv_nsec < NSEC_PER_SEC
    }

    /// 获取当前进程所有的cpu时间
    pub fn process_cputime_now() -> Self {
        let task = current_task().unwrap();
        let time = task.process_cputime();
        TimeSpec {
            tv_sec: time.as_secs() as usize,
            tv_nsec: time.subsec_nanos() as usize,
        }
    }

    /// 获取当前线程的cpu时间
    pub fn thread_cputime_now() -> Self {
        let task = current_task().unwrap();
        let time_data = task.get_time_data();
        let time = time_data.get_system_time() + time_data.get_user_time();
        TimeSpec {
            tv_sec: time.as_secs() as usize,
            tv_nsec: time.subsec_nanos() as usize,
        }
    }

    /// 获取开机时间
    pub fn boottime_now() -> Self {
        let task = current_task().unwrap();
        let time = task.get_time_data().get_machine_start_time();
        TimeSpec {
            tv_sec: time.as_secs() as usize,
            tv_nsec: time.subsec_nanos() as usize,
        }
    }
}

impl From<TimeSpec> for Duration {
    fn from(ts: TimeSpec) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

impl Display for TimeSpec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "tv_sec = {}, tv_nsec = {}", self.tv_sec, self.tv_nsec)
    }
}