use super::TimeVal;
use crate::sync::time_duration;
use core::{
    fmt::{Display, Formatter},
    time::Duration,
};

///
/// it_value ────────▶ (触发信号)
///                  it_interval ────▶ (再次触发)
///                                  it_interval ───▶ ..

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct ITimerVal {
    /// 定时器的周期性间隔时间（秒 + 微秒）
    /// - 当定时器首次到期后，后续将按此间隔重复触发
    /// - 若两个字段均为 0，表示定时器仅触发一次后停止
    /// - 对应 `setitimer()` 中 `itimerval.it_interval` 的语义
    pub it_interval: TimeVal,

    /// 定时器的初始到期时间（秒 + 微秒）
    /// - 表示第一次触发前的等待时间
    /// - 若两个字段均为 0，表示立即禁用定时器
    /// - 对应 `setitimer()` 中 `itimerval.it_value` 的语义
    pub it_value: TimeVal,
}

impl Display for ITimerVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ITimerVal {{ it_interval: {}, it_value: {} }}",
            self.it_interval, self.it_value
        )
    }
}

pub struct ItimerHelp {
    pub it_interval: Duration,
    pub it_value: Duration,
}

impl From<ITimerVal> for ItimerHelp {
    fn from(value: ITimerVal) -> Self {
        Self {
            it_interval: Duration::from(value.it_interval),
            it_value: Duration::from(value.it_value),
        }
    }
}
