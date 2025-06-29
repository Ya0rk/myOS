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