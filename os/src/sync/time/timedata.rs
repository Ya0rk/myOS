use core::time::Duration;
use crate::sync::timer::{time_duration, TIME_SLICE_DUATION};

pub struct TimeData {
    /// 开机时间
    machine_start_time: Duration,

    /// 用户态花费时间
    user_time: Duration,
    /// 内核态花费时间
    system_time: Duration,
    
    /// 在内核中调度进入processor开始执行的时间
    sched_in_time: Duration,
    /// 在内核中调度进程退出processor的时间
    sched_out_time: Duration,
    /// user trap 到 kernel的时间
    trap_in_time: Duration,
    /// kernel trap return 到 user的时间
    trap_out_time: Duration,

    child_user_time: Duration,
    child_system_time: Duration,
}

impl TimeData {
    pub fn new() -> Self {
        let machine_start_time = time_duration();
        TimeData {
            machine_start_time,
            user_time: Duration::ZERO,
            system_time: Duration::ZERO,
            sched_in_time: Duration::ZERO,
            sched_out_time: Duration::ZERO,
            trap_in_time: Duration::ZERO,
            trap_out_time: Duration::ZERO,
            child_user_time: Duration::ZERO,
            child_system_time: Duration::ZERO,
        }
    }

    /// 在task调度进processor时更新时间
    pub fn set_sched_in_time(&mut self) {
        let now = time_duration();
        self.sched_in_time = now;
    }

    /// 在task调度出processor时更新时间，增加内核态时间
    pub fn set_sched_out_time(&mut self) {
        let now = time_duration();
        self.sched_out_time = now;
        self.system_time += now - self.sched_in_time; // 相当于加上 调度的一个时间片
    }

    /// 在trap in时更新task的时间，增加用户态时间
    pub fn set_trap_in_time(&mut self) {
        let now = time_duration();
        self.trap_in_time = now;
        self.user_time += now - self.trap_out_time;
    }

    /// 在trap out时更新task的时间，增加内核态时间
    pub fn set_trap_out_time(&mut self) {
        let now = time_duration();
        self.trap_out_time = now;
        self.system_time += now - self.trap_in_time;
    }

    /// 在现成退出时更新user time和system time
    pub fn update_child_time_when_exit(&mut self, utime: Duration, stime: Duration) {
        self.child_user_time += utime;
        self.child_system_time += stime;
    }

    /// 获取用户态花费时间
    #[inline(always)]
    pub fn get_user_time(&self) -> Duration {
        self.user_time
    }
    /// 获取内核态花费时间
    #[inline(always)]
    pub fn get_system_time(&self) -> Duration {
        self.system_time
    }

    pub fn get_ustime(&self) -> (Duration, Duration) {
        (self.get_user_time(), self.get_system_time())
    }

    /// 判断任务在executor中的时间片是否用完
    pub fn usedout_timeslice(&self) -> bool {
        time_duration() - self.sched_in_time >= TIME_SLICE_DUATION
    }
}