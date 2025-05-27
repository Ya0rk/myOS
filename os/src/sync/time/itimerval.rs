use super::TimeVal;

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