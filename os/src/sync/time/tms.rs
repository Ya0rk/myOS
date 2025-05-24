use zerocopy::{Immutable, IntoBytes};
use crate::{sync::timer::get_time_ms, task::current_task};

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
        let task = current_task().unwrap();
        let timedata = task.get_time_data();
        let (utime, stime) = timedata.get_ustime();
        let (cutime, cstime) = timedata.get_child_ustime();
        Self {
            tms_utime: utime.as_micros() as clock_t,
            tms_stime: stime.as_micros() as clock_t,
            tms_cutime: cutime.as_micros() as clock_t,
            tms_cstime: cstime.as_micros() as clock_t,
        }
    }
}