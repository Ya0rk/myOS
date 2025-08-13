use super::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags};
use crate::{
    fs::{FileMeta, InodeMeta, InodeType}, hal::config::PIPE_BUFFER_SIZE, mm::page::Page, sync::{get_waker, once::LateInit, SpinNoIrqLock}, utils::{Errno, SysResult}
};
use alloc::boxed::Box;
use alloc::{
    collections::vec_deque::VecDeque,
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use async_trait::async_trait;
use core::{
    cmp::min,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use log::info;
use spin::Mutex;

pub struct Pipe {
    pub metadata: FileMeta,
    pub other: LateInit<Weak<Pipe>>,
    pub is_reader: bool,
    pub buffer: Arc<SpinNoIrqLock<PipeInner>>,
}

impl Pipe {
    /// make pipe: read end and wrtie end
    /// 创建一个管道并返回管道的读端和写端 (read_end, write_end)
    pub fn new() -> (Arc<Self>, Arc<Self>) {
        let buffer = Arc::new(SpinNoIrqLock::new(PipeInner::new()));
        let read_end = Arc::new(Self::read_end_with_buffer(buffer.clone()));
        let write_end = Arc::new(Self::write_end_with_buffer(buffer));
        read_end.other.init(Arc::downgrade(&write_end));
        write_end.other.init(Arc::downgrade(&read_end));

        (read_end, write_end)
    }
    /// 创建管道的读端
    pub fn read_end_with_buffer(buffer: Arc<SpinNoIrqLock<PipeInner>>) -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_RDONLY,
                DummyInode::new(InodeType::Fifo, "pipefile_readend"),
            ),
            other: LateInit::new(),
            is_reader: true,
            buffer,
        }
    }
    /// 创建管道的写端
    pub fn write_end_with_buffer(buffer: Arc<SpinNoIrqLock<PipeInner>>) -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_WRONLY,
                DummyInode::new(InodeType::Fifo, "pipefile_writeend"),
            ),
            // flags: OpenFlags::O_WRONLY,
            other: LateInit::new(),
            is_reader: false,
            buffer,
        }
    }
    /// 判断当前pipe的缓冲区是否满
    pub fn is_full(&self) -> bool {
        self.buffer.lock().buf.len() < PIPE_BUFFER_SIZE
    }
    /// 判断对方是否存活,没有的话代表已经关闭通道
    pub fn other_alive(&self) -> bool {
        self.other.strong_count() != 0
    }
    /// 唤醒读者
    pub fn wake_readers(&self, inner: &mut PipeInner) {
        while let Some(reader) = inner.reader_waker.pop_front() {
            reader.wake();
        }
    }
    /// 唤醒写者
    pub fn wake_writers(&self, inner: &mut PipeInner) {
        while let Some(writer) = inner.writer_waker.pop_front() {
            writer.wake();
        }
    }
    pub fn with_mut_buffer<T>(&self, f: impl FnOnce(&mut PipeInner) -> T) -> T {
        f(&mut self.buffer.lock())
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        let mut inner = self.buffer.lock();
        if self.is_reader {
            while let Some(waker) = inner.writer_waker.pop_front() {
                waker.wake();
            }
        } else {
            while let Some(waker) = inner.reader_waker.pop_front() {
                waker.wake();
            }
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
    pub buf: VecDeque<u8>,
    pub reader_waker: VecDeque<Waker>,
    pub writer_waker: VecDeque<Waker>,
    pub status: RingBufferStatus,
}

impl PipeInner {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            reader_waker: VecDeque::new(),
            writer_waker: VecDeque::new(),
            status: RingBufferStatus::Empty,
        }
    }

    /// 获取管道中剩余可读长度
    /// 需要比较用户buf还可以读多少数据，以及现在还剩多少数据
    pub fn available_read(&self, userbuf_left: usize) -> usize {
        return min(userbuf_left, self.buf.len());
    }
    /// 获取管道中剩余可写长度
    /// 判断用户还有多少数据要写，以及现在pipe还剩余多少空间
    pub fn available_write(&self, userbuf_left: usize) -> usize {
        return min(userbuf_left, PIPE_BUFFER_SIZE - self.buf.len());
    }
}

#[async_trait]
impl FileTrait for Pipe {
    fn metadata(&self) -> &FileMeta {
        &self.metadata
    }
    fn set_flags(&self, _flags: OpenFlags) {}

    async fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().readable());
        if buf.len() == 0 {
            return Ok(0);
        }
        PipeReadFuture {
            pipe: self,
            userbuf: buf,
            cur: 0,
        }
        .await
    }
    async fn write(&self, buf: &[u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().writable());
        if buf.len() == 0 {
            return Ok(0);
        }
        PipeWriteFuture {
            pipe: self,
            userbuf: buf,
            cur: 0,
        }
        .await
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_nlink = 1;
        stat.st_size = PIPE_BUFFER_SIZE as i64;
        stat.st_blksize = 512;
        stat.st_blocks = (PIPE_BUFFER_SIZE / 512) as i64;
        Ok(())
    }

    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
    /// 异步管道（Pipe）读取操作的核心逻辑，用于检查管道是否可读（有数据可读或对端已关闭），
    /// 并根据情况注册 Waker 以便在数据到达时唤醒异步任务。
    async fn pollin(&self) -> SysResult<bool> {
        if !self.buffer.lock().buf.is_empty() || self.other.strong_count() == 0 {
            return Ok(true);
        }

        // 还没有数据，此时等待被唤醒
        let waker = get_waker().await;
        self.buffer.lock().reader_waker.push_back(waker);
        Ok(false)
    }
    async fn pollout(&self) -> SysResult<bool> {
        if self.other.strong_count() == 0 {
            return Err(Errno::EPIPE);
        } else if !self.is_full() {
            // 缓冲区没有满代表可写
            return Ok(true);
        }

        let waker = get_waker().await;
        self.buffer.lock().writer_waker.push_back(waker);
        Ok(false)
    }
}

struct PipeReadFuture<'a> {
    pipe: &'a Pipe,
    userbuf: &'a mut [u8],
    cur: usize, // 记录当前用户数据buf读取到的位置
}

impl Future for PipeReadFuture<'_> {
    type Output = SysResult<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("[poll read future] start");
        let this = unsafe { self.get_unchecked_mut() };
        let userbuf_left = this.userbuf.len() - this.cur;
        let read_size = {
            let mut inner = this.pipe.buffer.lock();
            inner.available_read(userbuf_left)
        };

        if read_size > 0 {
            let mut inner = this.pipe.buffer.lock();
            let target = &mut this.userbuf[this.cur..this.cur + read_size];
            for (i, byte) in inner.buf.drain(..read_size).enumerate() {
                target[i] = byte;
            }
            this.cur += read_size;
            this.pipe.wake_writers(&mut inner);
            Poll::Ready(Ok(read_size))
        } else if !this.pipe.other_alive() {
            info!("[poll read future] other end closed");
            return Poll::Ready(Ok(0));
        } else {
            let mut inner = this.pipe.buffer.lock();
            inner.reader_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct PipeWriteFuture<'a> {
    pipe: &'a Pipe,
    userbuf: &'a [u8],
    cur: usize, // 记录当前用户buf已经写入的数据位置
}

impl Future for PipeWriteFuture<'_> {
    type Output = SysResult<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        info!("[pipe write future] start");
        if !this.pipe.other_alive() {
            info!("[pipe write future] read end closed");
            return Poll::Ready(Err(Errno::EPIPE));
        }

        let write_size = {
            let mut inner = this.pipe.buffer.lock();
            let userbuf_left = this.userbuf.len() - this.cur;
            inner.available_write(userbuf_left)
        };

        if write_size > 0 {
            let mut inner = this.pipe.buffer.lock();
            inner
                .buf
                .extend(&this.userbuf[this.cur..this.cur + write_size]);
            this.cur += write_size;
            this.pipe.wake_readers(&mut inner);
            Poll::Ready(Ok(write_size))
        } else {
            let mut inner = this.pipe.buffer.lock();
            inner.writer_waker.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct DummyInode(pub InodeMeta);

impl DummyInode {
    pub fn new(_type: InodeType, abspath: &str) -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            _type,
            0,
            abspath.into(),
        )))
    }
}

impl InodeTrait for DummyInode {
    fn metadata(&self) ->  &InodeMeta {
        &self.0
    }
}