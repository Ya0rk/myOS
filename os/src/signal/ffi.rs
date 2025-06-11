pub const MAX_SIGNUM: usize = 64;

#[allow(non_camel_case_types)]
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

    // 子进程发送给父进程的信号
    /// 子进程正常退出(调用 exit() 或从 main 返回)
    CLD_EXITED = 1,
    /// 子进程被信号杀死(SIGKILL、SIGTERM)
    CLD_KILLED = 2,
    /// 子进程被信号杀死并生成核心转储(如 SIGSEGV、SIGABRT)
    CLD_DUMPED = 3,
    /// 子进程被调试器捕获（触发断点或单步执行）
    CLD_TRAPPED = 4,
    /// 子进程被暂停(SIGSTOP、SIGTSTP)
    CLD_STOPPED = 5,
    /// 子进程已继续
    CLD_CONTINUED = 6,
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
            _ => todo!(),
        }
    }
}

bitflags! {
    /// 信号错误码(部分与 errno 共享)，0表示没有err（例如信号由 kill() 手动发送）
    #[derive(Clone, Copy)]
    pub struct SigErr: i32 {
        const EIO    = 5;  // I/O 错误（如磁盘读写失败触发的信号）
        const EACCES = 13; // 权限不足（如内存访问权限错误触发的 SIGSEGV）
        const EFAULT = 14; // 内存访问错误（如 SIGSEGV 的非法地址
        const EINVAL = 22; // 无效参数（如某些系统调用触发的信号）
        const EPIPE  = 32; // 管道破裂（SIGPIPE 的典型错误码）
    }

    #[derive(Clone, Copy, Debug)]
    pub struct SigMask: usize {
        const SIGHUP    = 1 << (1 - 1);
        const SIGINT    = 1 << (2 - 1);
        const SIGQUIT   = 1 << (3 - 1);
        const SIGILL    = 1 << (4 - 1);
        const SIGTRAP   = 1 << (5 - 1);
        const SIGABRT   = 1 << (6 - 1);
        const SIGBUS    = 1 << (7 - 1);
        const SIGFPE    = 1 << (8 - 1);
        const SIGKILL   = 1 << (9 - 1);
        const SIGUSR1   = 1 << (10 - 1);
        const SIGSEGV   = 1 << (11 - 1);
        const SIGUSR2   = 1 << (12 - 1);
        const SIGPIPE   = 1 << (13 - 1);
        const SIGALRM   = 1 << (14 - 1);
        const SIGTERM   = 1 << (15 - 1);
        const SIGSTKFLT = 1 << (16 - 1);
        const SIGCHLD   = 1 << (17 - 1);
        const SIGCONT   = 1 << (18 - 1);
        const SIGSTOP   = 1 << (19 - 1);
        const SIGTSTP   = 1 << (20 - 1);
        const SIGTTIN   = 1 << (21 - 1);
        const SIGTTOU   = 1 << (22 - 1);
        const SIGURG    = 1 << (23 - 1);
        const SIGXCPU   = 1 << (24 - 1);
        const SIGXFSZ   = 1 << (25 - 1);
        const SIGVTALRM = 1 << (26 - 1);
        const SIGPROF   = 1 << (27 - 1);
        const SIGWINCH  = 1 << (28 - 1);
        const SIGIO     = 1 << (29 - 1);
        const SIGPWR    = 1 << (30 - 1);
        const SIGSYS    = 1 << (31 - 1);
        /* --- other realtime signals --- */
        const   SIGTIMER    = 1 << (32 - 1);
        const   SIGCANCEL   = 1 << (33 - 1);
        const   SIGSYNCCALL = 1 << (34 - 1);
        const   SIGRT_3     = 1 << (35 - 1);
        const   SIGRT_4     = 1 << (36 - 1);
        const   SIGRT_5     = 1 << (37 - 1);
        const   SIGRT_6     = 1 << (38 - 1);
        const   SIGRT_7     = 1 << (39 - 1);
        const   SIGRT_8     = 1 << (40 - 1);
        const   SIGRT_9     = 1 << (41 - 1);
        const   SIGRT_10    = 1 << (42 - 1);
        const   SIGRT_11    = 1 << (43 - 1);
        const   SIGRT_12    = 1 << (44 - 1);
        const   SIGRT_13    = 1 << (45 - 1);
        const   SIGRT_14    = 1 << (46 - 1);
        const   SIGRT_15    = 1 << (47 - 1);
        const   SIGRT_16    = 1 << (48 - 1);
        const   SIGRT_17    = 1 << (49 - 1);
        const   SIGRT_18    = 1 << (50 - 1);
        const   SIGRT_19    = 1 << (51 - 1);
        const   SIGRT_20    = 1 << (52 - 1);
        const   SIGRT_21    = 1 << (53 - 1);
        const   SIGRT_22    = 1 << (54 - 1);
        const   SIGRT_23    = 1 << (55 - 1);
        const   SIGRT_24    = 1 << (56 - 1);
        const   SIGRT_25    = 1 << (57 - 1);
        const   SIGRT_26    = 1 << (58 - 1);
        const   SIGRT_27    = 1 << (59 - 1);
        const   SIGRT_28    = 1 << (60 - 1);
        const   SIGRT_29    = 1 << (61 - 1);
        const   SIGRT_30    = 1 << (62 - 1);
        const   SIGRT_31    = 1 << (63 - 1);
        const   SIGMAX      = 1 << (64 - 1);
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct SigActionFlag : usize {
        /// 当子进程停止时（如收到 `SIGSTOP`），不向父进程发送 `SIGCHLD` 信号。
        /// 通常用于避免不必要的子进程状态通知。
        const SA_NOCLDSTOP = 1;

        /// 当子进程终止时，不将其转换为僵尸进程（自动回收资源）。
        /// 相当于隐式调用 `signal(SIGCHLD, SIG_IGN)`。
        const SA_NOCLDWAIT = 2;

        /// 使用扩展信号处理函数（`sa_sigaction` 而非 `sa_handler`）。
        /// 允许信号处理函数接收额外的信号信息（`siginfo_t`）和上下文（`ucontext_t`）。
        const SA_SIGINFO = 4;

        /// 使用由 `sigaltstack` 设置的备用信号栈执行信号处理函数。
        /// 防止默认栈溢出时无法处理信号（如 `SIGSEGV`）。
        const SA_ONSTACK = 0x08000000;

        /// 自动重启被信号中断的系统调用（如 `read`、`write`）。
        /// 避免因信号导致系统调用返回 `EINTR` 错误。
        const SA_RESTART = 0x10000000;

        /// 在执行信号处理函数期间，不自动阻塞当前信号。
        /// 允许信号处理函数被同一信号递归中断（慎用）。
        const SA_NODEFER = 0x40000000;

        /// 信号处理函数执行完毕后，自动恢复为默认行为（`SIG_DFL`）。
        /// 通常用于一次性信号处理（如清理操作）。
        const SA_RESETHAND = 0x80000000;

        /// （Linux 特有）指定信号处理完成后的上下文恢复函数。
        /// 通常由 Glibc 内部使用，用户代码无需手动设置。
        const SA_RESTORER = 0x04000000;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(i32)]
pub enum SigNom {
    // 标准信号常量定义（基于 Linux/x86 架构）
    // 注：信号编号可能因操作系统或架构略有不同，此处以 Linux 常规值为准
    NOSIG = 0,      // 没有信号
    SIGHUP = 1,     // 终端挂起/控制进程终止
    SIGINT = 2,     // 键盘中断 (Ctrl+C)
    SIGQUIT = 3,    // 键盘退出 (Ctrl+\), 产生核心转储
    SIGILL = 4,     // 非法指令
    SIGTRAP = 5,    // 跟踪/断点陷阱（调试用）
    SIGABRT = 6,    // 进程调用 abort() 终止
    SIGBUS = 7,     // 总线错误（内存访问对齐错误等）
    SIGFPE = 8,     // 算术异常（除零、溢出等）
    SIGKILL = 9,    // 强制终止信号（不可捕获或忽略）
    SIGUSR1 = 10,   // 用户自定义信号 1
    SIGSEGV = 11,   // 段错误（无效内存访问）
    SIGUSR2 = 12,   // 用户自定义信号 2
    SIGPIPE = 13,   // 管道破裂（写入无读端的管道）
    SIGALRM = 14,   // alarm() 定时器超时
    SIGTERM = 15,   // 终止信号（可捕获的优雅退出）
    SIGSTKFLT = 16, // 协处理器栈错误（历史遗留，现代系统未使用）
    SIGCHLD = 17,   // 子进程状态改变（终止/暂停/恢复）
    SIGCONT = 18,   // 恢复已暂停的进程
    SIGSTOP = 19,   // 暂停进程（不可捕获或忽略）
    SIGTSTP = 20,   // 终端暂停 (Ctrl+Z)
    SIGTTIN = 21,   // 后台进程尝试读取终端
    SIGTTOU = 22,   // 后台进程尝试写入终端
    SIGURG = 23,    // 套接字紧急数据到达
    SIGXCPU = 24,   // 超出 CPU 时间限制
    SIGXFSZ = 25,   // 超出文件大小限制
    SIGVTALRM = 26, // 虚拟定时器超时
    SIGPROF = 27,   // 性能分析定时器超时
    SIGWINCH = 28,  // 终端窗口大小改变
    SIGIO = 29,     // 异步 I/O 事件（文件描述符就绪）
    SIGPWR = 30,    // 电源故障（UPS 电池低电量）
    SIGSYS = 31,    // 无效系统调用
    SIGTIMER = 32,  // 定时器信号（某些系统如 Solaris 使用）
    SIGCANCEL = 33, // 取消信号（某些系统如 Solaris 使用）
    SIGRTMAX = 64,  // 最大实时信号编号（实际可用信号可能更少）
}

impl SigMask {
    pub fn insert_sig(&mut self, sig_num: usize) {
        self.insert((SigMask::from_bits(1 << (sig_num - 1))).unwrap());
    }

    pub fn unset_sig(&mut self, sig_num: usize) {
        self.remove((SigMask::from_bits(1 << (sig_num - 1))).unwrap());
    }

    pub fn have(&self, sig_num: usize) -> bool {
        self.contains(SigMask::from_bits(1 << (sig_num - 1)).unwrap())
    }
}

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

#[allow(non_camel_case_types)]
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SigHandlerType {
    /// 恢复信号的默认行为
    DEFAULT = 0,
    /// 忽略该信号（如防止 SIGCHLD 产生僵尸进程）
    IGNORE = 1,
    /// 用户自定义的信号处理函数
    Customized { handler: usize },
}

impl SigHandlerType {
    pub fn default(sig: SigNom) -> Self {
        match sig {
            SigNom::SIGCHLD | SigNom::SIGURG | SigNom::SIGWINCH => Self::IGNORE,
            _ => Self::DEFAULT,
        }
    }
}

impl From<usize> for SigNom {
    fn from(value: usize) -> Self {
        if value <= MAX_SIGNUM {
            let ret: SigNom = unsafe { core::mem::transmute(value as i32) };
            return ret;
        } else {
            panic!("signal nomber out of bounds!");
        }
    }
}

pub const SIGBLOCK: usize = 0;
pub const SIGUNBLOCK: usize = 1;
pub const SIGSETMASK: usize = 2;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct LinuxSigInfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub _pad: [i32; 29],
    _align: [u64; 0],
}

impl LinuxSigInfo {
    pub fn new(signo: i32, code: i32) -> Self {
        Self {
            si_signo: signo,
            si_errno: 0,
            si_code: code,
            _pad: [0; 29],
            _align: [0; 0],
        }
    }
}
