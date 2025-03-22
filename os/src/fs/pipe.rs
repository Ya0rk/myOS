/// # 管道模块
/// `os/src/fs/pipe.rs`
/// ```
/// pub struct Pipe
/// enum RingBufferStatus
/// pub struct PipeRingBuffer
///
/// pub fn make_pipe()
/// ```
//
use super::{FileTrait, Kstat};
use crate::{mm::UserBuffer, utils::SysResult};
use alloc::{string::String, sync::{Arc, Weak}};
use spin::Mutex;
use async_trait::async_trait;
use alloc::boxed::Box;
use crate::task::suspend_current_and_run_next;

/// ### 管道
/// 由 读 `readable` / 写 `writable` 权限和 缓冲区 `buffer` 组成，用以分别表示管道的写端和读端
/// ```
/// pub fn read_end_with_buffer
/// pub fn write_end_with_buffer
/// ```
pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    /// make pipe: read end and wrtie end
    /// 创建一个管道并返回管道的读端和写端 (read_end, write_end)
    pub fn new() -> (Arc<Self>, Arc<Self>) {
        let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
        let read_end = Arc::new(Self::read_end_with_buffer(buffer.clone()));
        let write_end = Arc::new(Self::write_end_with_buffer(buffer.clone()));
        buffer.lock().set_write_end(&write_end);
        (read_end, write_end)
    }
    /// 创建管道的读端
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    /// 创建管道的写端
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
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

const RING_BUFFER_SIZE: usize = 32;

/// ### 管道缓冲区(双端队列,向右增长)
/// |成员变量|描述|
/// |--|--|
/// |`arr`|缓冲区内存块|
/// |`head`|队列头，读|
/// |`tail`|队列尾，写|
/// |`status`|队列状态|
/// |`write_end`|保存了它的写端的一个弱引用计数，<br>在需要确认该管道所有的写端是否都已经被关闭时，<br>通过这个字段很容易确认这一点|
/// ```
/// pub fn new()
/// pub fn set_write_end()
/// pub fn write_byte()
/// pub fn read_byte()
/// pub fn available_read()
/// pub fn available_write()
/// pub fn all_write_ends_closed()
/// ```
pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    write_end: Option<Weak<Pipe>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }
    /// 写一个字节到管道尾
    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::Normal;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }
    /// 从管道头读一个字节
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::Normal;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::Empty;
        }
        c
    }
    /// 获取管道中剩余可读长度
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::Empty {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }
    /// 获取管道中剩余可写长度
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::Full {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    /// 通过管道缓冲区写端弱指针判断管道的所有写端都被关闭
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

#[async_trait]
impl FileTrait for Pipe {
    fn readable(&self) -> SysResult<bool> {
        Ok(self.readable)
    }
    fn writable(&self) -> SysResult<bool> {
        Ok(self.writable)
    }
    async fn read(&self, buf: UserBuffer) -> SysResult<usize> {
        assert!(self.readable()?);
        let mut buf_iter = buf.into_iter();
        let mut read_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                if ring_buffer.all_write_ends_closed() {
                    return Ok(read_size);
                }
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // read at most loop_read bytes
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe {
                        *byte_ref = ring_buffer.read_byte();
                    }
                    read_size += 1;
                } else {
                    return Ok(read_size);
                }
            }
        }
    }
    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        assert!(self.writable()?);
        let mut buf_iter = buf.into_iter();
        let mut write_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref });
                    write_size += 1;
                } else {
                    return Ok(write_size);
                }
            }
        }
    }
    
    fn get_name(&self) -> SysResult<String> {
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
}
