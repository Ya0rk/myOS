use alloc::collections::VecDeque;
use crate::task::TaskStatus;
use super::ffi::{SigCode, SigErr, SigMask, SigNom};

/// 使用优先队列和普通队列
/// 
/// https://segmentfault.com/a/1190000044899251
pub struct SigPending {
    /// 检测哪些sig已经在队列中,避免重复加入队列
    mask: SigMask,
    /// 普通队列
    fifo: VecDeque<SigInfo>,
    /// 存放 SIGSEGV, SIGBUS, SIGILL, SIGTRAP, SIGFPE, SIGSYS
    prio: VecDeque<SigInfo>,
    /// 如果遇到的信号也在need_wake中，那就唤醒task
    pub need_wake: SigMask,
}

/// kill发送信号其实就是生成SigInfo然后加入对应task的SigPending中
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SigInfo {
    pub signo:   SigNom,  // 信号编号
    pub sigcode: SigCode, // 信号来源码
    pub sigerr:  SigErr,  // 错误码
    pub sifields: SigDetails, // 附加信息
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum SigDetails {
    Kill {
        pid: usize, // 发送信号的进程ID
        uid: usize, // 发送信号的用户ID, 目前不用管
    },

    Chld {
        pid: usize,         // 终止的子进程ID
        status: TaskStatus, // 子进程退出时的状态
        exit_code: i32,     // 退出码
    },

    None
}

lazy_static! {
    /// 优先级较高的信号
    static ref PRIO_SIG: SigMask = SigMask::SIGSEGV | SigMask::SIGBUS
    | SigMask::SIGILL | SigMask::SIGTRAP | SigMask::SIGFPE | SigMask::SIGSYS;
}

impl SigPending {
    pub fn new() -> Self {
        Self {
            mask: SigMask::empty(),
            fifo: VecDeque::new(),
            prio: VecDeque::new(),
            need_wake: SigMask::empty(),
        }
    }

    /// 取出一个信号，优先从prio队列获取
    pub fn take_one(&mut self) -> Option<SigInfo> {
        let sig_info = 
                self.prio
                .pop_front()
                .or_else(|| self.fifo.pop_front());
        match sig_info {
            Some(one) => {
                self.mask.unset_sig(one.signo as usize);
                return Some(one);
            }
            None => return sig_info,
        }
    }

    /// 用于wait4中取出SIGCHLD信号，所以只需要遍历fifo队列
    pub fn take_expected_one(&mut self, expect: SigMask) -> Option<SigInfo> {
        let intersection = self.mask & expect;
        if intersection.is_empty() {
            return None;
        }
        for i in 0..self.fifo.len() {
            let signo = self.fifo[i].signo as usize;
            if intersection.have(signo) {
                self.mask.unset_sig(signo);
                return self.fifo.remove(i);
            }
        }
        None
    }

    pub fn add(&mut self, siginfo: SigInfo) {
        let signo = siginfo.signo as usize;
        if !self.mask.have(signo) {
            self.mask.set_sig(signo);
            match PRIO_SIG.have(signo) {
                true  => self.prio.push_back(siginfo),
                false => self.fifo.push_back(siginfo),
            }
        }
    }

    pub fn len(&self) -> usize {
        self.fifo.len() + self.prio.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fifo.len() + self.prio.len() == 0
    }
}

impl SigInfo {
    pub fn new(
        si_signo: SigNom, 
        si_code: SigCode, 
        si_err: SigErr, 
        fields: SigDetails
    ) -> Self {
        Self {
            signo: si_signo,
            sigcode: si_code,
            sigerr: si_err,
            sifields: fields
        }
    }
}