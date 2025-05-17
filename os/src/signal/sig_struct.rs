use super::{SigActionFlag, SigHandlerType, SigMask, SigNom, MAX_SIGNUM, SIG_DFL, SIG_IGN};

/// 表示信号相应的处理方法,一共64个信号
#[derive(Clone, Copy)]
pub struct SigStruct {
    pub actions: [KSigAction; MAX_SIGNUM],
}

/// 内核层信号动作
#[derive(Clone, Copy)]
pub struct KSigAction {
    pub sa: SigAction,
    pub sa_type: SigHandlerType,
}

/// 用户层信号处理配
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SigAction {
    /// 信号处理函数类型，可能是自定义，也可能是默认
    pub sa_handler: usize,
    /// 控制信号处理行为的标志位
    pub sa_flags: SigActionFlag,
    pub sa_restorer: usize,
    /// 在执行信号处理函数期间临时阻塞的信号集合
    /// - 信号处理函数执行时，内核会自动将 sa_mask 中的信号添加到进程的阻塞信号集中
    /// - 处理函数返回后，阻塞信号集恢复为原状态
    pub sa_mask: SigMask,
}

impl SigStruct {
    pub fn new() -> Self {
        Self {
            actions: core::array::from_fn(|signo| KSigAction::new(SigNom::from(signo + 1))),
        }
    }

    /// 遍历所有信号，检查其当前处理方式：
    /// - 如果信号是 默认行为（SIG_DFL） 或 忽略（SIG_IGN）：保持不变。
    /// - 如果信号是 自定义处理函数：强制重置为 SIG_DFL。
    /// 避免新的进程信号处理函数被劫持
    pub fn flash_signal_handlers(&mut self) {
        self.actions.iter_mut().enumerate().for_each(|(i, ksa)| {
            if let SigHandlerType::Customized { .. } = ksa.sa_type {
                ksa.sa.sa_handler = match SigNom::from(i + 1) {
                    SigNom::SIGCHLD | SigNom::SIGURG | SigNom::SIGWINCH => SIG_IGN,
                    _ => SIG_DFL,
                };
                ksa.sa_type = SigHandlerType::default(SigNom::from(i + 1));
            }
        });
    }

    /// 获取信号对应的处理函数
    pub fn fetch_signal_handler(&self, signo: usize) -> KSigAction {
        if signo > MAX_SIGNUM {
            panic!("fetch_sighandler: signo out of bounds!");
        }
        self.actions[signo - 1]
    }

    /// 自定义设置信号处理动作
    pub fn set_action(&mut self, signo: usize, kaction: KSigAction) {
        self.actions[signo-1] = kaction;
    }
}

impl KSigAction {
    pub fn new(signo: SigNom) -> Self {
        Self {
            sa: SigAction::new(signo),
            sa_type: SigHandlerType::default(signo),
        }
    }
}

impl SigAction {
    pub fn new(signo: SigNom) -> Self {
        let sa_handler = match signo {
            SigNom::SIGCHLD | SigNom::SIGURG | SigNom::SIGWINCH => SIG_IGN,
            _ => SIG_DFL,
        };
        Self {
            sa_handler,
            sa_flags: SigActionFlag::empty(),
            sa_restorer: 0,
            sa_mask: SigMask::empty(),
        }
    }
}