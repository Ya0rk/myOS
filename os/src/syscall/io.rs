// this file is used for ppoll and pselect6
use core::{ptr::NonNull, time::Duration};
use alloc::{sync::Arc, vec::Vec};
use log::info;
use crate::{
    fs::FileTrait, signal::SigMask, 
    sync::{TimeSpec, TimeoutFuture}, 
    syscall::{ffi::{PollEvents, PollFd}, io_async::PpollFutrue}, 
    task::{current_task, TaskControlBlock}, utils::{Errno, SysResult}
};

// 新增 Guard 结构体, 在函数返回时自动回复Sigmask为original
struct SigMaskGuard {
    task: Arc<TaskControlBlock>,
    original_mask: Option<SigMask>,
}

impl SigMaskGuard {
    fn new(task: Arc<TaskControlBlock>, new_mask: Option<SigMask>) -> Self {
        let original_mask = task.get_blocked().clone();
        if let Some(mask) = new_mask {
            task.set_blocked(mask);
        }
        Self { task, original_mask: Some(original_mask) }
    }
}

impl Drop for SigMaskGuard {
    fn drop(&mut self) {
        if let Some(mask) = &self.original_mask {
            self.task.set_blocked(*mask);
        }
    }
}

/// wait for some event on a file descriptor
/// ppoll可以选择使用的信号屏蔽字。
/// 若sigmask为空，那么在与信号有关的方面，ppoll的运行状况和poll相同。
/// 否则，sigmask指向一信号屏蔽字，在调用ppoll时，以原子操作的方式安装该信号屏蔽字。
/// 在返回时恢复以前的信号屏蔽字。
/// fds: 传入传出参数，指向struct pollfd类型数组的首元素，每个数组元素指定一个描述符以及对其关心的状态
/// nfds：指明fds指向的数组元素个数
/// tmo_p：该参数指定ppoll阻塞等待文件描述符就绪的时间
/// sigmask: 临时替换进程的信号掩码（阻塞的信号集合），系统调用完成后恢复原掩码
/// 
/// 若在 tmo_p 指定的时间内无事件发生，定时器触发唤醒进程，返回 0
/// 否则返回就绪的文件描述符数量
/// 
/// The field fd contains a file descriptor for an open file.  If this
/// field is negative, then the corresponding events field is ignored
/// and the revents field returns zero.
pub async fn sys_ppoll(
    fds: usize,
    nfds: usize,
    tmo_p: usize,
    sigmask: usize,
) -> SysResult<usize> {
    // info!("[sys_ppoll] start fds: {}, nfds: {}, tmo_p: {}, sigmask: {}", fds, nfds, tmo_p, sigmask);
    if fds == 0 { return Err(Errno::EFAULT); }
    let task = current_task().unwrap();
    // 使用 Guard 管理信号掩码, 函数结束时自动回复sigmask
    let sigmask_guard = {
        let new_sigmask = if sigmask != 0 {
            Some(SigMask::from_bits(sigmask).ok_or(Errno::EINVAL)?)
        } else {
            None
        };
        SigMaskGuard::new(task.clone(), new_sigmask)
    };

    let time_out = match tmo_p {
        0 => None, 
        _ => {
            let timespec = unsafe{ *(tmo_p as *const TimeSpec) };
            Some(Duration::from(timespec))
        }
    };

    let myfds = unsafe{ core::slice::from_raw_parts(fds as *const PollFd, nfds) };
    let mut file_event = Vec::<(Arc<dyn FileTrait>, PollEvents)>::with_capacity(nfds);
    for item in myfds.iter() {
        let event = item.events;
        let fd = item.fd as usize;
        let file = task.get_file_by_fd(fd).ok_or(Errno::EBADF)?;
        file_event.push((file, event));
    }

    // 生成异步等待任务，检测文件状态是否可读或可写，根据PollEvents决定用户希望的状态
    let ppoll = PpollFutrue::new(file_event, fds);

    // 将其加入定时任务中
    let res = match time_out {
        None => ppoll.await,
        Some(timeout) => {
            let timelimit_task = TimeoutFuture::new(ppoll, timeout);
            match timelimit_task.await {
                Ok(res) => {
                    res
                }
                Err(_) => {
                    // 代表任务超时
                    info!("[sys_ppoll] task time out");
                    Ok(0)
                }
            }
        }
    };

    res
}

/// control device
pub fn sys_ioctl(fd: usize, op: usize, arg: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    if fd > task.fd_table_len() { return Err(Errno::EBADF); }
    Ok(0)
}


pub fn sys_pselect() -> SysResult<usize> {
    info!("[sys_pselect] start.");
    

    Ok(0)
}