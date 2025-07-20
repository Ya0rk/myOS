// this file is used for ppoll and pselect6
use alloc::{sync::Arc, vec::Vec};
use core::{intrinsics::unlikely, ptr::NonNull, time::Duration};
use hashbrown::{HashMap, HashSet};
use log::info;

use crate::{
    signal::SigMask,
    sync::{TimeSpec, TimeoutFuture},
    syscall::{
        ffi::{PollEvents, PollFd},
        io_async::{FdSet, IoFutrue, FD_PER_BITS, FD_SET_SIZE},
    },
    task::{current_task, TaskControlBlock},
    utils::{Errno, SysResult},
};

use super::io_async::UptrFmt;

// 新增 Guard 结构体, 在函数返回时自动回复Sigmask为original
pub struct SigMaskGuard {
    pub task: Arc<TaskControlBlock>,
    pub original_mask: Option<SigMask>,
}

impl SigMaskGuard {
    pub fn new(task: Arc<TaskControlBlock>, new_mask: Option<SigMask>) -> Self {
        let original_mask = task.get_blocked().clone();
        if let Some(mask) = new_mask {
            task.set_blocked(mask | original_mask);
        }
        Self {
            task,
            original_mask: Some(original_mask),
        }
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
pub async fn sys_ppoll(fds: usize, nfds: usize, tmo_p: usize, sigmask: usize) -> SysResult<usize> {
    // info!("[sys_ppoll] start fds: {}, nfds: {}, tmo_p: {}, sigmask: {}", fds, nfds, tmo_p, sigmask);
    if fds == 0 {
        return Err(Errno::EFAULT);
    }
    let task = current_task().unwrap();
    // 使用 Guard 管理信号掩码, 函数结束时自动回复sigmask
    let sigmask_guard = {
        let new_sigmask = if sigmask != 0 {
            let sigmask = sigmask as *const usize;
            Some(SigMask::from_bits(unsafe { *sigmask }).ok_or(Errno::EINVAL)?)
        } else {
            None
        };
        SigMaskGuard::new(task.clone(), new_sigmask)
    };

    let time_out = match tmo_p {
        0 => None,
        _ => {
            let timespec = unsafe { *(tmo_p as *const TimeSpec) };
            Some(Duration::from(timespec))
        }
    };

    let myfds = unsafe { core::slice::from_raw_parts(fds as *const PollFd, nfds) };
    let mut file_events = myfds.to_vec();

    // 生成异步等待任务，检测文件状态是否可读或可写，根据PollEvents决定用户希望的状态
    let ppoll = IoFutrue::new(file_events, UptrFmt::PollFds(fds));

    // 将其加入定时任务中
    let res = match time_out {
        None => ppoll.await,
        Some(timeout) => {
            let timelimit_task = TimeoutFuture::new(ppoll, timeout);
            match timelimit_task.await {
                Ok(res) => res,
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
    info!(
        "[sys_ioctl] start fd: {}, op: {:#x}, arg: {:#x}",
        fd, op, arg
    );
    if unlikely(arg == 0) {
        return Err(Errno::EFAULT);
    }
    let task = current_task().unwrap();
    if unlikely(fd > task.fd_table_len()) {
        return Err(Errno::EBADF);
    }
    // Ok(0)
    if let Some(file) = task.get_file_by_fd(fd) {
        if file.is_pipe() {
            return Ok(0);
        }
        if !file.is_deivce() {
            return Ok(0);
        }
        file.get_inode().ioctl(op, arg)
    } else {
        Err(Errno::EBADF)
    }
}

/// pselect主要目的是监视一组文件描述符的状态变化，
/// 并在任何一个文件描述符准备好读、写或有异常时返回。 readfds，
/// writefds，exceptfds：这些是指向文件描述符集的指针，
/// 分别用于监视读、 写和异常条件。 timeout：
/// 指向TimeSpec结构的指针，用于指定等待的超时时间。 sigmask：
/// 指向信号掩码的指针，用于在pselect调用期间阻止特定的信号。
pub async fn sys_pselect(
    nfds: usize,
    readfds_ptr: usize,
    writefds_ptr: usize,
    exceptfds_ptr: usize,
    timeout: usize,
    sigmask: usize,
) -> SysResult<usize> {
    info!("[sys_pselect] start.");
    let mut get_fdset = |ptr: usize| match ptr {
        0 => None,
        _ => Some(unsafe { &mut *(ptr as *mut FdSet) }),
    };
    let mut readfds = get_fdset(readfds_ptr);
    let mut writefds = get_fdset(writefds_ptr);
    let mut exceptfds = get_fdset(exceptfds_ptr);

    let timeout = match timeout {
        0 => None,
        _ => {
            let time = unsafe { *(timeout as *const TimeSpec) };
            Some(Duration::from(time))
        }
    };
    info!("[sys_pselect] nfds = {}, readfds = {:?}, writefds = {:?}, exceptfds = {:?}, timeout = {:?}",
            nfds, readfds, writefds, exceptfds, timeout);

    let mut file_events: Vec<PollFd> = Vec::new();
    let task = current_task().unwrap();
    // 遍历所有的fd，直到nfds，判断是否存在readfds或writefds、exceptfds中，如果在就加入fds中
    for fd in 0..FD_SET_SIZE {
        if fd >= nfds {
            break;
        }
        let fd_slot = fd / FD_PER_BITS;
        let offset = fd % FD_PER_BITS;

        let mut find_and_push = |set: &FdSet, event: PollEvents| {
            if set.isset(fd_slot, offset) {
                if let Some(pollfd) = file_events.last_mut()
                    && pollfd.fd as usize == fd
                {
                    pollfd.events |= event;
                } else {
                    task.fd_table.lock().get_file_by_fd(fd)?;
                    // println!("now fd = {}", fd);
                    let new = PollFd::new(fd as i32, event);
                    file_events.push(new);
                }
            }
            Ok(())
        };
        if let Some(readfds) = readfds.as_ref() {
            find_and_push(readfds, PollEvents::POLLIN)?;
        }
        if let Some(writefds) = writefds.as_ref() {
            find_and_push(writefds, PollEvents::POLLOUT)?;
        }
        if let Some(exceptfds) = exceptfds.as_ref() {
            find_and_push(exceptfds, PollEvents::POLLPRI)?;
        }
    }

    // 将其清空
    if let Some(fds) = readfds.as_mut() {
        fds.clr();
    }
    if let Some(fds) = writefds.as_mut() {
        fds.clr();
    }
    if let Some(fds) = exceptfds.as_mut() {
        fds.clr();
    }

    // 使用 Guard 管理信号掩码, 函数结束时自动回复sigmask
    let sigmask_guard = {
        let new_sigmask = if sigmask != 0 {
            let sigmask = sigmask as *const usize;
            Some(SigMask::from_bits(unsafe { *sigmask }).ok_or(Errno::EINVAL)?)
        } else {
            None
        };
        SigMaskGuard::new(task.clone(), new_sigmask)
    };

    // println!("[sys_pselect] readfds: {:#x}, writefds: {:#x}, exceptfds: {:#x}",
    // readfds_ptr, writefds_ptr, exceptfds_ptr);
    let iofuture = IoFutrue::new(
        file_events,
        UptrFmt::Pselect([readfds_ptr, writefds_ptr, exceptfds_ptr]),
    );

    match timeout {
        None => return iofuture.await,
        Some(span) => {
            let timeoutFuture = TimeoutFuture::new(iofuture, span);
            match timeoutFuture.await {
                Ok(res) => return res,
                Err(_) => return Ok(0),
            }
        }
    }
}
