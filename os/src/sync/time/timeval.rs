use zerocopy::{Immutable, IntoBytes};
use crate::sync::timer::{get_time_s, get_time_us};

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