use core::{future::Future, pin::Pin, task::Poll};
use alloc::{sync::Arc, vec::Vec};
use log::info;
use crate::{fs::FileTrait, utils::{Errno, SysResult}};
use super::ffi::{PollEvents, PollFd};


pub struct PpollFutrue {
    pub file_event: Vec<(Arc<dyn FileTrait>, PollEvents)>,
    pub user_fds_ptr: usize,
}

impl PpollFutrue {
    pub fn new(file_event: Vec<(Arc<dyn FileTrait>, PollEvents)>, user_fds_ptr: usize) -> Self {
        Self {
            file_event,
            user_fds_ptr
        }
    }
}

impl Future for PpollFutrue {
    type Output = SysResult<usize>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let this = unsafe{ self.get_unchecked_mut() };
        let mut res_vec = Vec::new();
        for (file, event) in this.file_event.iter() {
            if event.contains(PollEvents::POLLIN) {
                info!("[PpollFuture] pollin");
                let mut pollin_future = file.pollin();
                let res = Pin::new(&mut pollin_future).poll(cx);
                match res {
                    core::task::Poll::Ready(a) => {
                        let mut revent = PollEvents::empty();
                        if a { revent |= PollEvents::POLLIN;  }
                        else { revent |= PollEvents::POLLERR; }
                        res_vec.push(revent);
                    },
                    core::task::Poll::Pending => return core::task::Poll::Ready(Err(Errno::EINVAL)),
                }
            }
            if event.contains(PollEvents::POLLOUT) {
                info!("[PpollFuture] pollout");
                let mut pollout_future = file.pollout();
                let res = Pin::new(&mut pollout_future).poll(cx);
                match res {
                    core::task::Poll::Ready(a) => {
                        let mut revent = PollEvents::empty();
                        if a { revent |= PollEvents::POLLOUT; }
                        else { revent |= PollEvents::POLLERR; }
                        res_vec.push(revent);
                    },
                    core::task::Poll::Pending => return core::task::Poll::Ready(Err(Errno::EINVAL)),
                }
            }
        }

        if res_vec.len() > 0 {
            let len = res_vec.len();
            let user_fds = unsafe{ core::slice::from_raw_parts_mut(this.user_fds_ptr as *mut PollFd, len) };
            for (i, revent) in res_vec.iter().enumerate() {
                user_fds[i].revents |= *revent;
            }
            return Poll::Ready(Ok(len));
        }

        return Poll::Pending;
    }
}