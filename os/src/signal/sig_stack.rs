use super::SigMask;
use crate::hal::trap::{context::GPRegs, TrapContext};

/// 信号栈是为信号处理程序执行提供的专用栈空间.它通常包含以下内容:
/// 1.信号上下文：这是信号处理程序运行时的上下文信息，包括所有寄存器的值、
/// 程序计数器（PC）、栈指针等。它使得信号处理程序可以访问到被中断的程序的状态，
/// 并且在处理完信号后能够恢复这个状态，继续执行原程序。
/// 2.信号信息（siginfo_t）：这个结构提供了关于信号的具体信息，如信号的来源、
/// 产生信号的原因等。
/// 3.调用栈帧：如果信号处理程序调用了其他函数，
/// 那么这些函数的栈帧也会被压入信号栈。每个栈帧通常包含了函数参数、
/// 局部变量以及返回地址。
/// 4.信号处理程序的返回地址：当信号处理程序完成执行后，
/// 系统需要知道从哪里返回继续执行，因此信号栈上会保存一个返回地址。
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SignalStack {
    /// 信号栈的基地址（栈指针），指向栈的起始位置
    pub ss_sp: usize,
    /// 标志位，常见的值包括：
    ///
    /// - SS_ONSTACK: 表示当前正在使用信号栈
    /// - SS_DISABLE: 表示禁用信号栈
    pub ss_flags: i32,
    /// 信号栈的大小（以字节为单位）
    pub ss_size: usize,
}

/// 用户级上下文结构，保存了进程的上下文信息
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UContext {
    pub uc_flags: usize,
    /// 当前上下文返回时将恢复执行的下一个上下文的指针
    pub uc_link: usize,
    // 当前上下文使用的栈信息,包含栈的基址、大小等信息
    pub uc_stack: SignalStack,
    // 当前上下文活跃时被阻塞的信号集
    pub uc_sigmask: SigMask,
    // don't know why, struct need to be exact the same with musl libc
    pub uc_sig: [usize; 16],
    // 保存具体机器状态的上下文信息，这是一个机器相关的表示，包含了处理器的寄存器状态等信息
    pub uc_mcontext: MContext,
}

/// 整个结构体大小为784字节，为了适配musl libc
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MContext {
    pub user_gp: GPRegs,
    pub fpstate: [usize; 66],
}

impl Default for SignalStack {
    fn default() -> Self {
        SignalStack {
            ss_sp: 0usize.into(),
            ss_flags: 0,
            ss_size: 0,
        }
    }
}

impl UContext {
    pub fn new(
        old_sigmask: SigMask,
        sig_stack: Option<SignalStack>,
        trap_cx: &TrapContext,
    ) -> Self {
        let mut user_gp: GPRegs = trap_cx.user_gp;
        // 将old sepc暂时存储在user_x[0]中,这个sepc用于sigreturn时恢复
        // 这里的sepc是信号处理函数的返回地址
        // user_x[0] = trap_cx.sepc;
        Self {
            uc_flags: 0,
            uc_link: 0,
            uc_stack: sig_stack.unwrap_or_default(),
            uc_sigmask: old_sigmask,
            uc_sig: [0; 16],
            uc_mcontext: MContext::new(user_gp),
        }
    }

    pub fn get_user_gp(&self) -> GPRegs {
        self.uc_mcontext.user_gp
    }
}

impl MContext {
    fn new(user_reg: GPRegs) -> Self {
        Self {
            user_gp: user_reg,
            fpstate: [0; 66],
        }
    }
}
