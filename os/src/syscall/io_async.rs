use super::ffi::{PollEvents, PollFd};
use crate::{
    fs::FileTrait,
    task::current_task,
    utils::{Errno, SysResult},
};
use alloc::{sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin, task::Poll};
use log::info;

pub struct IoFutrue {
    pub file_event: Vec<PollFd>,
    pub user_fds_ptr: UptrFmt,
}

impl IoFutrue {
    pub fn new(file_event: Vec<PollFd>, user_fds_ptr: UptrFmt) -> Self {
        Self {
            file_event,
            user_fds_ptr,
        }
    }
}

impl Future for IoFutrue {
    type Output = SysResult<usize>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut res_vec = Vec::new();
        let task = current_task().unwrap();
        for pollfd in this.file_event.iter_mut() {
            pollfd.revents = PollEvents::empty();
            let file = match task.get_file_by_fd(pollfd.fd as usize) {
                Some(f) => f,
                None => {
                    println!("[PpollFuture] fd {} not found", pollfd.fd);
                    continue;
                }
            };

            info!("[iofuture] fd = {}", pollfd.fd);
            if pollfd.events.contains(PollEvents::POLLIN) {
                match unsafe{Pin::new_unchecked(& mut file.pollin())}.poll(cx) {
                    Poll::Ready(Ok(ok)) => {
                        if ok {
                            pollfd.revents |= PollEvents::POLLIN;
                            res_vec.push(PollEvents::POLLIN);
                        }
                    }
                    Poll::Ready(Err(_)) => {
                        pollfd.revents |= PollEvents::POLLERR;
                        res_vec.push(PollEvents::POLLERR);
                    }
                    Poll::Pending => unreachable!()
                }
            }
            if pollfd.events.contains(PollEvents::POLLOUT) {
                match unsafe{Pin::new_unchecked(& mut file.pollin())}.poll(cx) {
                    Poll::Ready(Ok(ok)) => {
                        if ok {
                            pollfd.revents |= PollEvents::POLLOUT;
                            res_vec.push(PollEvents::POLLOUT);
                        }
                    }
                    Poll::Ready(Err(_)) => {
                        pollfd.revents |= PollEvents::POLLERR;
                        res_vec.push(PollEvents::POLLERR);
                    }
                    Poll::Pending => unreachable!()
                }
            }

            
        }

        if res_vec.len() > 0 {
            let len = res_vec.len();
            this.user_fds_ptr.update(&this.file_event)?;
            return Poll::Ready(Ok(len));
        }
        return Poll::Pending;
    }
}

/// 用于IoFuture中，ppoll和pselect都需要Iofuture，但是传入的参数不同
/// 所以需要UptrFmt进行统一
pub enum UptrFmt {
    PollFds(usize),
    Pselect([usize; 3]),
}

impl UptrFmt {
    /// 更新用户指针，对于已经完成操作的file，将位图置1，因为我们之前清除了位图
    /// 用户可以通过置1的位置判断哪些文件完成了操作
    pub fn update(&self, now_fds: &Vec<PollFd>) -> SysResult<()> {
        match self {
            Self::PollFds(user_fds_ptr) => {
                let user_fds = unsafe {
                    core::slice::from_raw_parts_mut((*user_fds_ptr) as *mut PollFd, now_fds.len())
                };
                user_fds.copy_from_slice(now_fds);
                Ok(())
            }
            Self::Pselect([readfds, writefds, exceptfds]) => {
                for pollfd in now_fds {
                    let flash = |ptr: usize, event: PollEvents| {
                        if ptr == 0 {
                            return Ok(());
                        }
                        let fdset = unsafe { &mut *(ptr as *mut FdSet) };
                        if pollfd.revents.contains(event) {
                            fdset.setfd(pollfd.fd as usize);
                        }
                        Ok(())
                    };
                    flash(*readfds, PollEvents::POLLIN)?;
                    flash(*writefds, PollEvents::POLLOUT)?;
                    flash(*exceptfds, PollEvents::POLLPRI)?;
                }
                Ok(())
            }
        }
    }
}

pub const FD_SET_SIZE: usize = 1024;
pub const FD_PER_BITS: usize = 8 * size_of::<usize>();
pub const FD_SET_LEN: usize = FD_SET_SIZE / FD_PER_BITS;

/// 这里使用位图实现1024个文件描述符
#[repr(C)]
#[derive(Debug)]
pub struct FdSet {
    /// 数组中每一个元素是一个位图，每一位都代表一个fd
    pub fd_list: [usize; FD_SET_LEN],
}

impl FdSet {
    /// 判断是否存在该fd，fd_solt是数组下标，offset是偏移
    pub fn isset(&self, fd_slot: usize, offset: usize) -> bool {
        self.fd_list[fd_slot] & (1 << offset) != 0
    }

    /// 清空
    pub fn clr(&mut self) {
        for i in 0..FD_SET_LEN {
            self.fd_list[i] = 0usize;
        }
    }

    pub fn setfd(&mut self, fd: usize) {
        if fd > FD_SET_SIZE {
            return;
        }
        let index = fd / FD_PER_BITS;
        let offset = fd % FD_PER_BITS;
        self.fd_list[index] |= (1 << offset);
    }
}
