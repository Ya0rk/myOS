use core::{cmp::min, future::Future, task::{Poll, Waker}};
use super::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags};
use crate::{hal::config::PIPE_BUFFER_SIZE, mm::{UserBuffer, page::Page}, sync::once::LateInit, utils::{Errno, SysResult}};
use alloc::{collections::vec_deque::VecDeque, string::String, sync::{Arc, Weak}, vec::Vec};
use spin::Mutex;
use async_trait::async_trait;
use alloc::boxed::Box;

pub struct Pipe {
    flags: OpenFlags,
    other : LateInit<Weak<Pipe>>,
    buffer: Arc<Mutex<PipeInner>>,
}

impl Pipe {
    /// make pipe: read end and wrtie end
    /// 创建一个管道并返回管道的读端和写端 (read_end, write_end)
    pub fn new() -> (Arc<Self>, Arc<Self>) {
        let buffer = Arc::new(Mutex::new(PipeInner::new()));
        let read_end  = Arc::new(Self::read_end_with_buffer(buffer.clone()));
        let write_end = Arc::new(Self::write_end_with_buffer(buffer));
        read_end.other.init(Arc::downgrade(&write_end));
        write_end.other.init(Arc::downgrade(&read_end));

        (read_end, write_end)
    }
    /// 创建管道的读端
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeInner>>) -> Self {
        Self {
            flags: OpenFlags::O_RDONLY,
            other: LateInit::new(),
            buffer,
        }
    }
    /// 创建管道的写端
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeInner>>) -> Self {
        Self {
            flags: OpenFlags::O_WRONLY,
            other: LateInit::new(),
            buffer,
        }
    }
}

/// 管道缓冲区状态
#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    Full,
    Empty,
    Normal,
}

pub struct PipeInner {
    arr: [u8; PIPE_BUFFER_SIZE],
    head: usize, // 指针开始位置
    tail: usize, // 指针结束位置
    reader_waker: VecDeque<Waker>,
    writer_waker: VecDeque<Waker>,
    status: RingBufferStatus,
}

impl PipeInner {
    pub fn new() -> Self {
        Self {
            arr: [0; PIPE_BUFFER_SIZE],
            head: 0,
            tail: 0,
            reader_waker: VecDeque::new(),
            writer_waker: VecDeque::new(),
            status: RingBufferStatus::Empty,
        }
    }
    /// 写n个字节到管道尾
    pub fn write_nbyte(&mut self, nbyte: &[u8]) {
        // 这里不用再判断是否不够用
        self.status = RingBufferStatus::Normal;

        for &c in nbyte {
            self.arr[self.tail] = c;
            self.tail = (self.tail + 1) % PIPE_BUFFER_SIZE;
        }

        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }
    /// 从管道头读n个字节
    pub fn read_nbyte(&mut self, nbyte: usize) -> Vec<u8> {
        // 这里不用再判断可读数量，因为len就是计算后的最小值，不会越界
        self.status = RingBufferStatus::Normal;
        let mut res = Vec::with_capacity(nbyte);

        for _ in 0..nbyte {
            res.push(self.arr[self.head]);
            self.head = (self.head + 1) % PIPE_BUFFER_SIZE;
        }

        if self.head == self.tail {
            self.status = RingBufferStatus::Empty;
        }
        res
    }
    /// 获取管道中剩余可读长度
    pub fn available_read(&self, buf_len: usize) -> usize {
        if self.status == RingBufferStatus::Empty {
            0
        } else if self.tail > self.head {
            min(buf_len, self.tail - self.head)
        } else {
            min(buf_len, PIPE_BUFFER_SIZE - self.head + self.tail)
        }
    }
    /// 获取管道中剩余可写长度
    pub fn available_write(&self, buf_len: usize) -> usize {
        if self.status == RingBufferStatus::Full {
            0
        } else if self.tail > self.head {
            min(buf_len, self.tail - self.head)
        } else {
            min(buf_len, PIPE_BUFFER_SIZE - self.head + self.tail)
        }
    }
}

#[async_trait]
impl FileTrait for Pipe {
    fn set_flags(&self, _flags: OpenFlags) {
        todo!()
    }
    fn get_flags(&self) -> OpenFlags {
        self.flags
    }
    fn readable(&self) -> bool {
        self.flags.contains(OpenFlags::O_RDONLY)
    }
    fn writable(&self) -> bool {
        self.flags.contains(OpenFlags::O_WRONLY)
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, buf: UserBuffer) -> SysResult<usize> {
        assert!(self.readable());
        if buf.len() == 0{
            return Ok(0);
        }
        PipeReadFuture {
            pipe: self,
            buf,
            cur: 0,
        }.await
    }
    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        assert!(self.writable());
        if buf.len() == 0{
            return Ok(0);
        }
        PipeWriteFuture {
            pipe: self,
            buf,
            cur: 0,
        }.await
    }
    
    fn get_name(&self) -> SysResult<String> {
        todo!()
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    // fn poll(&self, events: PollEvents) -> PollEvents {
    //     let mut revents = PollEvents::empty();
    //     if events.contains(PollEvents::IN) && self.readable {
    //         revents |= PollEvents::IN;
    //     }
    //     if events.contains(PollEvents::OUT) && self.writable {
    //         revents |= PollEvents::OUT;
    //     }
    //     let ring_buffer = self.inner_lock();
    //     if self.readable && ring_buffer.all_write_ends_closed() {
    //         revents |= PollEvents::HUP;
    //     }
    //     if self.writable && ring_buffer.all_read_ends_closed() {
    //         revents |= PollEvents::ERR;
    //     }
    //     revents
    // }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }

    fn is_dir(&self) -> bool {
        todo!()
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}

struct PipeReadFuture<'a> {
    pipe: &'a Pipe,
    buf: UserBuffer,
    cur: usize
}

impl Future for PipeReadFuture<'_> {
    type Output = SysResult<usize>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let mut inner = self.pipe.buffer.lock();
        let size = inner.available_read(self.buf.len() - self.cur);
        if size > 0 {
            let mut pos = 0;
            let mut idx = 0;
            loop {
                if pos >= size { break; }
                let len = min(self.buf[idx].len(), size - pos);
                self.buf[idx].copy_from_slice(&inner.read_nbyte(len));
                idx += 1;
                pos += len;
            }
            self.cur += size;
            while let Some(waker) = inner.writer_waker.pop_front() {
                waker.wake();
            }
            Poll::Ready(Ok(size))
        } else {
            if self.pipe.other.strong_count() == 0 {
                return Poll::Ready(Ok(0));
            }
            inner.reader_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct PipeWriteFuture<'a> {
    pipe: &'a Pipe,
    buf: UserBuffer,
    cur: usize
}

impl Future for PipeWriteFuture<'_> {
    type Output = SysResult<usize>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let mut inner = self.pipe.buffer.lock();
        if self.pipe.other.strong_count() == 0 {
            return Poll::Ready(Err(Errno::EPIPE));
        }
        let size = inner.available_write(self.buf.len() - self.cur);
        if size > 0 {
            let mut pos = 0;
            let mut idx = 0;
            loop {
                if pos >= size { break; }
                let len = min(self.buf[idx].len(), size - pos);
                inner.write_nbyte(&self.buf[idx][0..len]);
                idx += 1;
                pos += len;
            }
            self.cur += size;
            while let Some(waker) = inner.reader_waker.pop_front() {
                waker.wake();
            }
            Poll::Ready(Ok(size))
        } else {
            inner.writer_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}