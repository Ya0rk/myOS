mod timeval;
mod timesepc;
mod tms;
mod timedata;
mod timestamp;

pub use timeval::TimeVal;
pub use timesepc::TimeSpec;
pub use tms::Tms;
pub use timedata::TimeData;
pub use timestamp::TimeStamp;

/// 实时系统时钟，可能受系统时间调整影响 (例如NTP同步)
pub const CLOCK_REALTIME: usize = 0;

/// 单调递增时钟，不受系统时间调整影响，适合测量时间间隔
pub const CLOCK_MONOTONIC: usize = 1;

/// 进程消耗的CPU时间，统计当前进程在所有CPU核心上的执行时间
pub const CLOCK_PROCESS_CPUTIME_ID: usize = 2;

/// 线程消耗的CPU时间，统计当前线程在所有CPU核心上的执行时间
pub const CLOCK_THREAD_CPUTIME_ID: usize = 3;

/// 包含系统挂起时间的单调时钟，从系统启动开始计算
pub const CLOCK_BOOTTIME: usize = 7;