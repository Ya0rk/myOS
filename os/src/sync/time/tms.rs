use zerocopy::{Immutable, IntoBytes};
use crate::sync::timer::get_time_ms;

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