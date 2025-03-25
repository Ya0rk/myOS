use super::{SigActionFlag, SigHandler, SigMask, SigNom, MAX_SIGNUM};

/// 表示信号相应的处理方法,一共64个信号
#[derive(Clone)]
pub struct SigStruct {
    pub action: [KSigAction; MAX_SIGNUM],
}

/// 内核层信号动作
#[derive(Clone)]
pub struct KSigAction {
    pub sa: SigAction,
}

/// 用户层信号处理配
#[derive(Clone)]
pub struct SigAction {
    /// 信号处理函数类型，可能是自定义，也可能是默认
    pub sa_handler: SigHandler,
    /// 控制信号处理行为的标志位
    pub sa_flags: SigActionFlag,
    /// 在执行信号处理函数期间临时阻塞的信号集合
    /// - 信号处理函数执行时，内核会自动将 sa_mask 中的信号添加到进程的阻塞信号集中
    /// - 处理函数返回后，阻塞信号集恢复为原状态
    pub sa_mask: SigMask,
}

impl SigStruct {
    pub fn new() -> Self {
        Self {
            action: core::array::from_fn(|signo| KSigAction::new(SigNom::my_from_bits(signo + 1))),
        }
    }
}

impl KSigAction {
    pub fn new(signo: SigNom) -> Self {
        Self {
            sa: SigAction::new(signo),
        }
    }
}

impl SigAction {
    pub fn new(signo: SigNom) -> Self {
        Self {
            sa_handler: SigHandler::default(signo),
            sa_flags: SigActionFlag::empty(),
            sa_mask: SigMask::empty(),
        }
    }
}