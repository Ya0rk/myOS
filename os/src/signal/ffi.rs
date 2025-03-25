
pub const MAX_SIGNUM: usize = 64;

#[derive(Copy, Debug, Clone)]
#[repr(i32)]
pub enum SigCode {
    /// sent by kill, sigsend, raise
    User = 0,
    /// sent by kernel from somewhere
    Kernel = 0x80,
    /// 通过sigqueue发送
    Queue = -1,
    /// 定时器过期时发送
    Timer = -2,
    /// 当实时消息队列的状态发生改变时发送
    Mesgq = -3,
    /// 当异步IO完成时发送
    AsyncIO = -4,
    /// 信号因文件描述符就绪触发
    SigIO = -5,
    /// 信号由 tkill() 或 tgkill() 发送
    TKILL = -6,
}

impl From<i32> for SigCode {
    fn from(value: i32) -> Self {
        match value {
            0 => SigCode::User,
            0x80 => SigCode::Kernel,
            -1 => SigCode::Queue,
            -2 => SigCode::Timer,
            -3 => SigCode::Mesgq,
            -4 => SigCode::AsyncIO,
            -5 => SigCode::SigIO,
            -6 => SigCode::TKILL,
            _  => todo!()
        }
    }
}

bitflags! {
    pub struct SigNom: i32 {
        // 标准信号常量定义（基于 Linux/x86 架构）
        // 注：信号编号可能因操作系统或架构略有不同，此处以 Linux 常规值为准
        const NOSIG  = 0;       // 没有信号
        const SIGHUP = 1;       // 终端挂起/控制进程终止
        const SIGINT = 2;       // 键盘中断 (Ctrl+C)
        const SIGQUIT = 3;      // 键盘退出 (Ctrl+\), 产生核心转储
        const SIGILL = 4;       // 非法指令
        const SIGTRAP = 5;      // 跟踪/断点陷阱（调试用）
        const SIGABRT = 6;      // 进程调用 abort() 终止
        const SIGBUS = 7;       // 总线错误（内存访问对齐错误等）
        const SIGFPE = 8;       // 算术异常（除零、溢出等）
        const SIGKILL = 9;      // 强制终止信号（不可捕获或忽略）
        const SIGUSR1 = 10;     // 用户自定义信号 1
        const SIGSEGV = 11;     // 段错误（无效内存访问）
        const SIGUSR2 = 12;     // 用户自定义信号 2
        const SIGPIPE = 13;     // 管道破裂（写入无读端的管道）
        const SIGALRM = 14;     // alarm() 定时器超时
        const SIGTERM = 15;     // 终止信号（可捕获的优雅退出）
        const SIGSTKFLT = 16;   // 协处理器栈错误（历史遗留，现代系统未使用）
        const SIGCHLD = 17;     // 子进程状态改变（终止/暂停/恢复）
        const SIGCONT = 18;     // 恢复已暂停的进程
        const SIGSTOP = 19;     // 暂停进程（不可捕获或忽略）
        const SIGTSTP = 20;     // 终端暂停 (Ctrl+Z)
        const SIGTTIN = 21;     // 后台进程尝试读取终端
        const SIGTTOU = 22;     // 后台进程尝试写入终端
        const SIGURG = 23;      // 套接字紧急数据到达
        const SIGXCPU = 24;     // 超出 CPU 时间限制
        const SIGXFSZ = 25;     // 超出文件大小限制
        const SIGVTALRM = 26;   // 虚拟定时器超时
        const SIGPROF = 27;     // 性能分析定时器超时
        const SIGWINCH = 28;    // 终端窗口大小改变
        const SIGIO = 29;       // 异步 I/O 事件（文件描述符就绪）
        const SIGPWR = 30;      // 电源故障（UPS 电池低电量）
        const SIGSYS = 31;      // 无效系统调用
        const SIGTIMER = 32;    // 定时器信号（某些系统如 Solaris 使用）
        const SIGRTMAX = 64;    // 最大实时信号编号（实际可用信号可能更少）
    }

    /// 信号错误码(部分与 errno 共享)，0表示没有err（例如信号由 kill() 手动发送）
    pub struct SigErr: i32 {
        const EIO    = 5;  // I/O 错误（如磁盘读写失败触发的信号）
        const EACCES = 13; // 权限不足（如内存访问权限错误触发的 SIGSEGV）
        const EFAULT = 14; // 内存访问错误（如 SIGSEGV 的非法地址
        const EINVAL = 22; // 无效参数（如某些系统调用触发的信号）
        const EPIPE  = 32; // 管道破裂（SIGPIPE 的典型错误码）
    }

    pub struct SigMask: u64 {
        const NOSIG     = 1 << 0;
        const SIGHUP    = 1 << 1;
        const SIGINT    = 1 << 2;
        const SIGQUIT   = 1 << 3;
        const SIGILL    = 1 << 4;
        const SIGTRAP   = 1 << 5;
        const SIGABRT   = 1 << 6;
        const SIGBUS    = 1 << 7;
        const SIGFPE    = 1 << 8;
        const SIGKILL   = 1 << 9;
        const SIGUSR1   = 1 << 10;
        const SIGSEGV   = 1 << 11;
        const SIGUSR2   = 1 << 12;
        const SIGPIPE   = 1 << 13;
        const SIGALRM   = 1 << 14;
        const SIGTERM   = 1 << 15;
        const SIGSTKFLT = 1 << 16;
        const SIGCHLD   = 1 << 17;
        const SIGCONT   = 1 << 18;
        const SIGSTOP   = 1 << 19;
        const SIGTSTP   = 1 << 20;
        const SIGTTIN   = 1 << 21;
        const SIGTTOU   = 1 << 22;
        const SIGURG    = 1 << 23;
        const SIGXCPU   = 1 << 24;
        const SIGXFSZ   = 1 << 25;
        const SIGVTALRM = 1 << 26;
        const SIGPROF   = 1 << 27;
        const SIGWINCH  = 1 << 28;
        const SIGIO     = 1 << 29;
        const SIGPWR    = 1 << 30;
        const SIGSYS    = 1 << 31;
        /* --- other realtime signals --- */
        const   SIGTIMER    = 1 << 32;
        const   SIGCANCEL   = 1 << 33;
        const   SIGSYNCCALL = 1 << 34;
        const   SIGRT_3     = 1 << 35;
        const   SIGRT_4     = 1 << 36;
        const   SIGRT_5     = 1 << 37;
        const   SIGRT_6     = 1 << 38;
        const   SIGRT_7     = 1 << 39;
        const   SIGRT_8     = 1 << 40;
        const   SIGRT_9     = 1 << 41;
        const   SIGRT_10    = 1 << 42;
        const   SIGRT_11    = 1 << 43;
        const   SIGRT_12    = 1 << 44;
        const   SIGRT_13    = 1 << 45;
        const   SIGRT_14    = 1 << 46;
        const   SIGRT_15    = 1 << 47;
        const   SIGRT_16    = 1 << 48;
        const   SIGRT_17    = 1 << 49;
        const   SIGRT_18    = 1 << 50;
        const   SIGRT_19    = 1 << 51;
        const   SIGRT_20    = 1 << 52;
        const   SIGRT_21    = 1 << 53;
        const   SIGRT_22    = 1 << 54;
        const   SIGRT_23    = 1 << 55;
        const   SIGRT_24    = 1 << 56;
        const   SIGRT_25    = 1 << 57;
        const   SIGRT_26    = 1 << 58;
        const   SIGRT_27    = 1 << 59;
        const   SIGRT_28    = 1 << 60;
        const   SIGRT_29    = 1 << 61;
        const   SIGRT_30    = 1 << 62;
        const   SIGRT_31    = 1 << 63;
    }
}

impl SigMask {
    pub fn set_sig(&mut self, sig_num: i32) {
        *self |= (SigMask::from_bits(1 << sig_num)).unwrap();
    }

    pub fn unset_sig(&mut self, sig_num: i32) {
        *self -= (SigMask::from_bits(1 << sig_num)).unwrap();
    }

    pub fn have(&self, sig_num: i32) -> bool {
        self.contains(SigMask::from_bits(1 << sig_num).unwrap())
    }
}