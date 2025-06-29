#import "../components/prelude.typ": *
= 进程间通信

== 信号机制

信号是操作系统向进程传递事件通知的一种机制，主要用于通知进程发生了异步事件。在我们的内核中，严格按照Liunx中对于信号结构的设计，实现了相对完善且清晰的信号机制。 信号相关结构体设计自顶向下为分别为，SigStruct（一个包含所有信号处理方法的数组），其中每个元素为 KSigAction（内核层信号动作），SigAction（信号处理相关配置），三者关系如下：

#code-figure(
```rs
#[derive(Clone, Copy)]
pub struct SigStruct {
    pub actions: [KSigAction; MAX_SIGNUM],
}

/// 内核层信号动作
#[derive(Clone, Copy)]
pub struct KSigAction {
    pub sa: SigAction,
    pub sa_type: SigHandlerType,
}

/// 用户层信号处理配
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SigAction {
    /// 信号处理函数类型，可能是自定义，也可能是默认
    pub sa_handler: usize,
    /// 控制信号处理行为的标志位
    pub sa_flags: SigActionFlag,
    pub sa_restorer: usize,
    /// 在执行信号处理函数期间临时阻塞的信号集合
    /// 信号处理函数执行时，内核会自动将 sa_mask 中的信号添加到进程的阻塞
    /// 信号集
    /// 处理函数返回后，阻塞信号集恢复为原状态
    pub sa_mask: SigMask,
}
```,
    caption: [信号相关结构体定义],
    label-name: "signal-structs",
)

== 信号传输

在Del0n1x中，用户可以通过 kill 系统调用向进程传送信号，利用参数 pid 可以找到对应的进程或进程组，然后调用 TCB 成员函数接口 proc_recv_siginfo 将信号推入对应进程的待处理信号队列中（结构如下）：

#code-figure(
```rs
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
```,
    caption: [SigPending 结构体],
    label-name: "sigpending-struct",
)

#h(2em)我们将信号处理队列分为普通队列和优先队列，对不同的信号做了优先级处理，这样的数据结构时的Del0n1x对于紧急时间和高优先级时的相应延迟更低，提高了内核的实时性。

对于队列中的信号结构体 SigInfo设计，我们借鉴了Linux中的 siginfo_t 实现方式，同时对其进行了简化和封装，能够携带更多的数据信息（发送者pid、子进程exit code和信号编码等），这样极大的方便了 Wait4 和 do_signal 中对于不同信号的处理分发流程。

== 信号处理

进程因系统调用、中断或异常进入内核态，完成内核任务后，在返回用户态前，内核会检查该进程的未决信号。Del0n1x中信号处理集中在 do_signal 函数中，我们会依次遍历prio和fifo队列，如果该信号没有被阻塞，则根据 siginfo 中信号编码找到对应的信号处理函数 KSigAction，然后对 KSigAction 中的 sa_type 字段进行模式匹配，对应的动作分别为 Ignore(忽略该信号)、Default（系统默认处理）和Customized（用户自定义处理函数）。

对于用户自定义函数，内核会下面的流程进行处理（如下图）：

构建用户态栈帧：在内核栈中创建新栈帧，如果用户没有自定义栈帧位置，那么默认为将用户栈sp向低地址扩展分配，确保信号处理函数有独立栈空间。

修改返回上下文：将原用户态执行点（如 pc 寄存器）保存到UContex中，然后复制到用户栈；然后修改当前trap_context指向用户处理函数。

切换至用户态：跳转至信号处理函数入口并执行用户自定义函数。在这一过程中，为了避免信号的嵌套处理，需要将原信号加入屏蔽字。

切换到内核态：处理函数结束后调用 sigreturn() 系统调用，主动陷入内核态。
内核从用户栈恢复原进程上下文，清除信号屏蔽字。


// TODO: 需要自己画一个

#figure(
  image("assets/do_signal.png"),
  caption: [信号处理],
  supplement: [图],
)<信号处理>

== 管道 Pipe

每个进程有自己独立的地址空间，在用户态的情况下，任何一个进程的全局变量对于其他进程都是不可访问的，所以进程之间的数据交换必须通过内核，
内核为不同的进程之间开辟一个缓冲区，进程 A 将数据写入缓冲区，然后进程 B 从缓冲区中取走数据，Pipe 就是通过这样的机制实现进程间的通信。

=== Pipe 设计

从基本功能的分析得知，Pipe 需要维护一个写端和一个读端供进程操作缓冲区，同时，考虑到读写之间并发问题，我们需要对缓冲区加锁，也就是默认当
有一方在读或在写时，另一方需要阻塞；除此之外，为了解决缓冲区并没有数据但是读者持有锁的问题，我们还需要设计读写者唤醒机制，基于异步架构的
调度方式，Del01x 用数组保存进程 waker 句柄来记录有待唤醒的读写者。基于此，Del0n1x 的 Pipe 结构设计如下：

#code-figure(
    ```rust
    pub struct Pipe {
        pub flags: OpenFlags,
        pub other: LateInit<Weak<Pipe>>,
        pub is_reader: bool,
        pub buffer: Arc<SpinNoIrqLock<PipeInner>>,
    }
    pub struct PipeInner {
        pub buf: VecDeque<u8>,
        pub reader_waker: VecDeque<Waker>,
        pub writer_waker: VecDeque<Waker>,
        pub status: RingBufferStatus,
    }
    ```,
    caption: [Pipe 结构设计],
    label-name: "Pipe 结构设计",
)

#h(2em)鉴于在 Linux 中 Pipe 是一种文件，Del0n1x 为 Pipe 实现了 FileTrait，这样我们可以像操作文件一样建立和操控 Pipe。

=== 读者写者通信

在 Del0n1x 中，我们通过手写 Future 的方式实现管理读者写者之间同步。下面以读者为例，我们为读者实现 `PipeReadFuture` 记录
读者在异步轮循中的关键字段，解释如下。在读者访问缓冲区前，会计算目前可读的长度，如果缓冲区没有数据，那么读者会将自己的唤醒句柄
waker 保存在 pipe 的读者带唤醒数组中，等待写者完成数据写入后唤醒；如果缓冲区有数据，那么读者将该数据段拷贝到 userbuf 中，并通知
写者此时 Pipe 缓冲区有空余空间可写。


#code-figure(
    ```rust
    struct PipeReadFuture<'a> {
        /// 与写者通信的管道
        pipe: &'a Pipe,
        /// 用户空间指针
        userbuf: &'a mut [u8],
        /// 记录当前用户数据buf读取到的位置
        cur: usize, 
    }

    impl Future for PipeReadFuture<'_> {
        type Output = SysResult<usize>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
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
                return Poll::Ready(Ok(0));
            } else {
                let mut inner = this.pipe.buffer.lock();
                inner.reader_waker.push_back(cx.waker().clone());
                Poll::Pending
            }
        }
    }
    ```,
    caption: [读者同步],
    label-name: "读者同步",
)

#h(2em)这样的设计看似读者写者之间互相交错唤醒，场面和谐。但试想一下一种极端情况，如果读者或者写者在操作完缓冲区后，因为一些原因导致进程
退出，而此时 Pipe 中还有待唤醒的读者或写者，那这样会造成死等的情况。这样的 bug 是在适配 libcbench 的 ptread 中发现的，为了解决这样的问题，
需要 为 Pipe 实现 Drop 方式自动唤醒等待队列中的进程。


== System V IPC 机制

=== System V IPC 对象

Del0n1x 支持进程间通过System V IPC机制进行通信。System V IPC使用全局唯一的IPC Key标识 IPC 对象，知晓IPC Key的进程可以调用相关的ABI获取IPC对象，并使用相关的 ABI 创建通信信道。

System V IPC 对象包括三种类型的对象：
#list(
    [消息队列 Message Queue，msg],
    [信号量 Semaphore，sem],
    [共享内存 Shared Memory，shm],
    indent: 4em
)

#h(2em)Del0n1x 实现了 IPCPerm 结构体用于维护 IPC 对象的所有权和权限信息。在此基础上，Del0n1x 对共享内存对象提供了支持，并为其余两种类型的IPC对象预留了可供拓展的接口。

#code-figure(
```rust
#[repr(C)]
pub struct IPCPerm {
    pub key: IPCKey,
    pub uid: u32,
    pub gid: u32,
    pub cuid: u32,
    pub cgid: u32,
    pub mode: IPCPermMode,
    pub seq: u32,
}
```,
    caption: [IPC Perm结构体],
    label-name: "ipc-perm-struct"
)     

=== IPC Key管理器

Del0n1x 定义了一个全局的IPC Key管理器，为每一个IPC对象分配唯一的IPC Key。
#code-figure(
```rust
pub struct IPCKey(pub i32);
pub struct IPCKeyAllocator {
    current: i32,
    recycled: BTreeSet<i32>,
}
impl IPCKeyAllocator {
    /// 初始化分配器
    pub fn new() -> Self {...}
    /// 分配IPC Key
    pub fn alloc(&mut self) -> IPCKey {...}
    /// 释放IPC Key
    pub fn dealloc(&mut self, key: i32) {...}
}
```,
    caption: [IPC Key全局分配器],
    label-name: "ipc-key-allocator"
)           

#h(2em)进程可以通过传入`IPC_PRIVATE`调用分配器为创建的 IPC 对象分配IPC Key，也可以指定对象的IPC Key，以便从IPC对象管理器中获取IPC Key对应的IPC对象。

#code-figure(
```rust
impl IPCKey {
    pub fn new_alloc() -> IPCKey {
        IPC_KEY_ALLOCATOR.lock().alloc()
    }
    pub fn from_user(user_key: i32) -> IPCKey {
        const IPC_PRIVATE: i32 = 0;
        if (user_key == IPC_PRIVATE) {
            Self::new_alloc()
        } else {
            IPCKey(user_key)
        }
    }
}
```,
    caption: [IPC Key的创建与获取],
    label-name: "ipc-key-init"
)

=== System V 共享内存

Del0n1x 实现了`ShmidDs`和`ShmObj`数据结构，作为操作 System V 共享内存的句柄。用户进程可以使用 `shmget`、`shmctl`、`shmat`、`shmdt`等 System V 共享内存相关 ABI 创建、访问共享内存 IPC 对象，并通过映射和读写 System V 共享内存实现通信。

#code-figure(
```rust
/// System V共享内存对象元数据
pub struct ShmidDs {
    pub shm_perm: IPCPerm,
    pub shm_segsz: usize,
    pub shm_atime: usize,
    pub shm_dtime: usize,
    pub shm_ctime: usize,
    pub shm_cpid: usize,
    pub shm_lpid: usize,
    pub shm_nattch: usize,
}
/// 维护System V共享内存的映射目标
pub struct ShmObject {
    pub shmid_ds: ShmidDs,
    pub pages: Vec<Weak<Page>>,
}
```,
    caption: [System V共享内存IPC对象],
    label-name: "sysv-shm-obj"
)

#pagebreak()  // 强制分页