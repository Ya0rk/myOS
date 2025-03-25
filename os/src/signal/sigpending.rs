use crossbeam_queue::SegQueue;
use crate::task::TaskStatus;

use super::ffi::{SigCode, SigErr, SigMask, SigNom};

pub struct SigPending {
    pub queue: SignalQueue,
}

/// 使用优先队列和普通队列
/// 
/// https://segmentfault.com/a/1190000044899251
pub struct SignalQueue {
    /// 检测哪些sig已经在队列中,避免重复加入队列
    pub mask: SigMask,
    /// 普通队列
    pub fifo: SegQueue<SigInfo>,
    /// 存放 SIGSEGV, SIGBUS, SIGILL, SIGTRAP, SIGFPE, SIGSYS
    pub prio: SegQueue<SigInfo>,
}

pub struct SigInfo {
    pub signo:   SigNom,  // 信号编号
    pub sigcode: SigCode, // 信号来源码
    pub sigerr:  SigErr,  // 错误码
    pub sifields: SigDetails, // 附加信息
}

#[repr(C)]
pub enum SigDetails {
    Kill {
        pid: usize, // 发送信号的进程ID
        uid: usize, // 发送信号的用户ID, 目前不用管
    },

    Chld {
        pid: usize,         // 终止的子进程ID
        status: TaskStatus, // 子进程退出时的状态
        exit_code: usize,   // 退出码
    }
}

