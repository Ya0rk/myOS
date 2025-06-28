#import "../template.typ": img, tbl
#import "../algorithm.typ": algorithm-figure
#import "@preview/lovelace:0.2.0": *
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
    /// - 信号处理函数执行时，内核会自动将 sa_mask 中的信号添加到进程的阻塞信号集中
    /// - 处理函数返回后，阻塞信号集恢复为原状态
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

我们将信号处理队列分为普通队列和优先队列，对不同的信号做了优先级处理，这样的数据结构时的Del0n1x对于紧急时间和高优先级时的相应延迟更低，提高了内核的实时性。

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
    image("../assets/do_signal.png", width: 90%),
)<leaderboard>