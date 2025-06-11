mod itimerval;
mod timedata;
mod timesepc;
mod timestamp;
mod timeval;
mod tms;

pub use itimerval::ITimerVal;
pub use itimerval::ItimerHelp;
pub use timedata::TimeData;
pub use timesepc::TimeSpec;
pub use timestamp::TimeStamp;
pub use timeval::TimeVal;
pub use tms::Tms;

/// 实时系统时钟，可能受系统时间调整影响 (例如NTP同步)
pub const CLOCK_REALTIME: usize = 0;
/// 单调递增时钟，不受系统时间调整影响，适合测量时间间隔
pub const CLOCK_MONOTONIC: usize = 1;
/// 进程消耗的CPU时间，统计当前进程在所有CPU核心上的执行时间
pub const CLOCK_PROCESS_CPUTIME_ID: usize = 2;
/// 线程消耗的CPU时间，统计当前线程在所有CPU核心上的执行时间
pub const CLOCK_THREAD_CPUTIME_ID: usize = 3;
/// 获取较为粗糙的时间
pub const CLOCK_REALTIME_COARSE: usize = 5;
/// 包含系统挂起时间的单调时钟，从系统启动开始计算
pub const CLOCK_BOOTTIME: usize = 7;

pub const TIMER_ABSTIME: usize = 1;

pub const UTIME_NOW: usize = 0x3fffffff;
pub const UTIME_OMIT: usize = 0x3ffffffe;

/// 用于setitimer和getitimer系统调用的定时器类型
/// 真实时间定时器，基于实际流逝的时间触发（例如挂钟时间）。
/// 超时时会发送 SIGALRM 信号。
pub const ITIMER_REAL: usize = 0;
/// 虚拟定时器，仅统计进程在用户态的执行时间。
/// 当进程在用户模式下消耗指定 CPU 时间后，会发送 SIGVTALRM 信号。
pub const ITIMER_VIRTUAL: usize = 1;
/// 统计分析定时器，统计进程在用户态和内核态的总执行时间。
/// 当用户态+内核态消耗指定 CPU 时间后，会发送 SIGPROF 信号。
pub const ITIMER_PROF: usize = 2;
