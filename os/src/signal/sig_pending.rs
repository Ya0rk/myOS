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

pub enum WhichQueue {
    Fifo,
    Prio,
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
    /// 参数mask：当前进程的信号掩码，不能取出掩码中的信号
    pub fn take_one(&mut self, mask: SigMask) -> Option<SigInfo> {
        if let Some(index) = self.prio.iter().position(|x| !mask.have(x.signo as usize)) {
            let siginfo = self.prio.remove(index).unwrap();
            self.mask.unset_sig(siginfo.signo as usize); // 这里的mask代表该信号是否在队列中，然后将其删去
            return Some(siginfo);
        }

        if let Some(index) = self.fifo.iter().position(|x| !mask.have(x.signo as usize)) {
            let siginfo = self.fifo.remove(index).unwrap();
            self.mask.unset_sig(siginfo.signo as usize);
            return Some(siginfo);
        }

        return None;
    }

    /// 用于wait4中取出SIGCHLD信号，所以只需要遍历fifo队列
    pub fn take_expected_one(&mut self, expect: SigMask) -> Option<SigInfo> {
        match self.has_expected(expect) {
            (true, i, q) => {
                let siginfo = match q {
                    WhichQueue::Fifo => self.fifo.remove(i as usize),
                    WhichQueue::Prio => self.prio.remove(i as usize),
                    WhichQueue::None => unimplemented!()
                };
                self.mask.unset_sig(siginfo.unwrap().signo as usize);
                return siginfo;
            },
            (false, _, _) => return None
        }
    }

    /// 获取信号，不会将其从队列删除
    pub fn get_expected_one(&self, expect: SigMask) -> Option<SigInfo> {
        match self.has_expected(expect) {
            (true, i, q) => {
                let siginfo = match q {
                    WhichQueue::Fifo => self.fifo.get(i as usize),
                    WhichQueue::Prio => self.prio.get(i as usize),
                    WhichQueue::None => unimplemented!()
                };
                return siginfo.cloned();
            },
            (false, _, _) => return None
        }
    }

    pub fn has_expected(&self, expect: SigMask) -> (bool, isize, WhichQueue) {
        let intersection = self.mask & expect;
        if intersection.is_empty() {
            return (false, -1, WhichQueue::None);
        }

        for i in 0..self.fifo.len() {
            let signo = self.fifo[i].signo as usize;
            if intersection.have(signo) {
                return (true, i as isize, WhichQueue::Fifo);
            }
        }

        for i in 0..self.prio.len() {
            let signo = self.prio[i].signo as usize;
            if intersection.have(signo) {
                return (true, i as isize, WhichQueue::Prio);
            }
        }

        return (false, -1, WhichQueue::None);
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