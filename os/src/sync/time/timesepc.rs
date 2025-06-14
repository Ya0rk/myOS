use crate::{
    sync::timer::{
        get_time_ms, get_time_ns, get_time_s, time_duration, MSEC_PER_SEC, NSEC_PER_SEC,
    },
    task::current_task,
};
use core::{fmt::Display, time::Duration};
use zerocopy::IntoBytes;

#[derive(Copy, Clone, IntoBytes, Debug, Default)]
#[repr(C)]
pub struct TimeSpec {
    /// 秒
    pub tv_sec: usize,
    /// 纳秒
    pub tv_nsec: usize,
}

impl TimeSpec {
    /// 返回当前时间
    pub fn new() -> Self {
        let tick = get_time_ms();
        let tv_sec = tick / MSEC_PER_SEC;
        let tv_nsec = (tick % 1000) * 1_000_000;

        TimeSpec { tv_sec, tv_nsec }
    }

    pub fn check_valid(&self) -> bool {
        self.tv_nsec < NSEC_PER_SEC && (self.tv_nsec as isize) >= 0 && (self.tv_sec as isize) >= 0
    }

    /// 获取当前进程所有的cpu时间
    pub fn process_cputime_now() -> Self {
        let task = current_task().unwrap();
        let time = task.process_cputime();
        println!(
            "[process_cputime_now] sec = {}, nsec = {}",
            time.as_secs() as usize,
            time.subsec_nanos() as usize
        );
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

    /// 获取较为粗糙的时间，这里使用秒
    pub fn get_coarse_time() -> Self {
        let ts = get_time_s();
        TimeSpec {
            tv_sec: ts,
            tv_nsec: 0,
        }
    }
}

impl From<TimeSpec> for Duration {
    fn from(ts: TimeSpec) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

impl From<Duration> for TimeSpec {
    fn from(value: Duration) -> Self {
        Self {
            tv_sec: value.as_secs() as usize,
            tv_nsec: value.subsec_nanos() as usize,
        }
    }
}

impl Display for TimeSpec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "tv_sec = {}, tv_nsec = {}", self.tv_sec, self.tv_nsec)
    }
}
