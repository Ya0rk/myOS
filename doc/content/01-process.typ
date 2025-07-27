#import "../components/prelude.typ": *

= 进程管理

== 概述

Del0n1x操作系统采用无栈协程作为核心任务管理模型，该设计基于用户级轻量级线程理念，允许任务在执行过程中挂起并恢复。
无栈协程与有栈协程的核心差异体现在上下文管理机制上：在rCore和xv6中，采用有栈方式进行任务切换，每个任务有独立栈空间，每次任务切换需要调用`__switch`函数，并在切换时保存完整栈帧及s0\~s11、ra、sp等14个RISC-V寄存器。
而Del0n1x的无栈协程不依赖独立栈结构，转而通过状态机管理上下文状态，任务挂起时仅需保存当前执行位置和关键状态变量，显著降低内存开销与切换延迟。

在传统操作系统中，进程调度需切换用户级上下文、寄存器上下文和系统级上下文（含内核栈与页表），这种完整上下文切换必须通过内核态系统调用完成。
但是Del0n1x采用共享内核栈架构，所有任务复用同一内核栈空间，任务切换时不涉及内核栈切换，这极大地降低了任务切换的开销。


== 任务调度

在无栈协程中，任务调度并不需要在栈上保存任务栈帧，而是将必要的中间信息保存为状态机，放置在堆空间进行保存，这样不仅减少了任务调度开销，同时提高了调度过程中的安全性，降低栈溢出的风险。

在Del0n1x中，当对异步函数调用`.await`方法时，async-task库会首次调用Future的poll方法，如果poll返回的结果是Pending，那么该任务将被挂起，await 将控制权交给调度器，以便另一个任务可以继续进行。任务调度如下图：

#figure(
  image("assets/sched.png"),
  caption: [任务调度],
  supplement: [图],
)<任务调度>

// === 异步编程并非"万能钥匙"

// 在初次接触异步调度机制时，我们曾天真地认为：既然异步调度效率如此之高，何不将所有函数都设计为异步形式？但随着对异步机制的深入理解，我们逐渐认识到这实际上陷入了一个典型的认知误区——将异步视为解决所有问题的"银弹"。

// 首先需要明确的是，异步编程的核心价值在于解决I/O密集型场景的性能瓶颈。在计算机系统中，I/O操作（如磁盘读写、网络通信）的延迟通常高达微秒(μs)至毫秒(ms)级别，而CPU指令的执行时间仅为纳秒(ns)级。异步I/O的优势在于允许CPU在等待慢速I/O操作完成期间，转而执行其他计算任务，从而显著提升系统吞吐量。然而，对于CPU密集型操作（如getpid()、sched_yield()等系统调用），它们本身不存在阻塞等待的情况，强制异步化不仅无法带来性能提升，反而会引入额外的调度开销。这样的背景下同步调用的直接返回机制在性能上远优于异步回调所需的上下文切换。

// 其次，异步编程带来了显著的状态管理复杂度。异步操作通常需要通过回调函数或Future机制来实现，开发者必须精心设计状态机来管理操作的生命周期。特别是在内核开发中，若要对poll阶段的代码流程进行精细控制，往往需要手动实现Future的poll轮询机制，这种复杂的状态管理不仅增加了代码的实现难度，更大大提升了调试和维护的成本。一个典型的内核异步实现往往需要处理：任务唤醒、资源竞争、错误恢复等多重状态，这使得系统稳定性的保障变得极具挑战性。

== 任务调度队列与执行器

在任务调度队列实现中，Del0n1x将调度队列分为 FIFO 和 PRIO 队列。感谢优秀作品 Plntry 对 async-task 库做出的 PR，
使我们在任务调度过程中可以获取调度信息。通过调度信息，我们可以对任务做出更加精细的控制。
如果任务在运行时被唤醒，则将其加入 FIFO 队列，其他的就放入 PRIO 队列进行管理。

#code-figure(
```rs
struct TaskQueue {
    normal: SpinNoIrqLock<VecDeque<Runnable>>,
    prior: SpinNoIrqLock<VecDeque<Runnable>>,
}

....
// 任务入队逻辑
if info.woken_while_running {
    queue.push_normal(runnable);
} else {
    queue.push_prior(runnable);
}

....
```,
    caption: [任务队列结构],
    label-name: "task-queue-struct",
)

#h(2em)在Del0n1x中，我们使用统一的 `TaskFuture` 封装了任务。

#code-figure(
```rust
pub enum TaskFuture<F: Future<Output = ()> + Send + 'static> {
    UserTaskFuture {
        task: Arc<TaskControlBlock>,
        future: F,
    },
    KernelTaskFuture {
        future: F,
    },
}
```,
    caption: [任务 Future 枚举],
    label-name: "task-future-enum"
)

#code-figure(
```rs
fn poll(
    self: Pin<&mut Self>,
    cx: &mut core::task::Context<'_>,
) -> core::task::Poll<Self::Output> {
    let this = unsafe { self.get_unchecked_mut() };

    match this {
        TaskFuture::UserTaskFuture { task, future } => {
            let processor = get_current_cpu();
            processor.user_task_checkin(task);  // 用户任务 checkin
            let ret = unsafe { Pin::new_unchecked(future).poll(cx) };
            processor.user_task_checkout(task); // 用户任务 checkout
            ret
        }
        TaskFuture::KernelTaskFuture { future } => {
            unsafe { Pin::new_unchecked(future).poll(cx) }
        }
    }
}
```,
    caption: [任务 Future poll 实现],
    label-name: "task-future-poll",
)

#h(2em)对于用户任务，在poll轮循中实现了任务的切换调度。当任务checkin时，需要在修改TCB的时间戳记录调度时间，然后切换CPU中运行任务和切换页表。当任务checkout时，需要判断浮点寄存器状态是否为dirty，以确定是否保存浮点寄存器，然后清空CPU当前任务，并记录任务checkout时间。 对于内核任务，Del0n1x并没有设计任务切换，而是让该任务一直poll，直到任务结束。这类任务主要是shell程序。

spawn_user_task可以设置一个用户任务。Del0n1x将用户任务的future设置为 `trap_loop` 循环，负责处理任务在用户态和内核态之间的切换，直到任务结束。执行 `executor::spawn(future)` 将任务挂入全局队列中等待被调度。

#code-figure(
```rs
pub fn spawn_user_task(user_task: Arc<TaskControlBlock>) {
    let future = TaskFuture::user_task(
        user_task.clone(), 
        trap_loop(user_task)
    );
    executor::spawn(future);
}
```,
    caption: [用户任务生成函数],
    label-name: "spawn-user-task",
)

== 多核心CPU管理

在Del0n1x中，我们将处理器抽象为CPU结构体，使用内核中CPU结构体进行统一管理。`current` 中保存当前正在运行的任务的 TCB。`timer_irq_cnt` 记录内核时钟中断次数，在内核时钟中断处理函数中会增加这个计数器，trap return时计数器会清零。如果计数器大于阈值，手动对该任务yield进行调度，避免任务一直占用CPU。`ktrap_ret` 用于记录 kernel trap处理结果。

#code-figure(
```rs
pub struct CPU {
    current: Option<Arc<TaskControlBlock>>,
    timer_irq_cnt: usize,
    hart_id: usize,
    ktrap_ret: Option<SysResult<()>>,
}
```,
    caption: [CPU 结构体定义],
    label-name: "cpu-struct",
)

#h(2em)单个CPU被存放在全局的`PROCESSORS`管理器中，并向外暴露接口，通过管理器我们能获取到当前任务的上下文信息和页表token、CPU id号等。

#code-figure(
```rs
const PROCESSOR: CPU = CPU::new();
pub static PROCESSORS: SyncProcessors = 
    SyncProcessors(UnsafeCell::new([PROCESSOR; HART_NUM]));
```,
    caption: [全局 CPU 管理器],
    label-name: "processors-global",
)

== 任务控制块

进程是操作系统中资源管理的基本单位，而线程是操作系统中调度的基本单位。由于在Linux设计理念中，线程是轻量级进程，所以在Del0n1x中使用统一的任务控制块（Task Control Block，TCB）来管理进程和线程。
同时我们对TCB字段进行细粒化的加锁处理，类似 `memory_space` 和 `trap_cx` 等高频访问的字段来说，可以显著减少并发过程中锁的竞争，提高并发效率。

#code-figure(
```rs
pub struct TaskControlBlock {
    pub pid: Pid,               // 任务标识符
    pub tgid: AtomicUsize,      // leader的pid号
    pub pgid: AtomicUsize,      // 进程组id
    pub task_status: SpinNoIrqLock<TaskStatus>,// 任务状态
    pub thread_group: Shared<ThreadGroup>,     // 线程组
    pub memory_space: Shared<MemorySpace>,     // 地址空间
    pub fd_table: Shared<FdTable>,             // 文件描述表
    pub current_path: Shared<String>,          // 路径
    pub robust_list: Shared<RobustList>,       // 存储线程的信息
    pub futex_list: Shared<FutexBucket>,       // futex互斥锁队列
    pub itimers: Shared<[ITimerVal; 3]>,       // 任务的内部时钟
    pub fsz_limit: Shared<Option<RLimit64>>,   // 任务的资源限制
    pub shmid_table: Shared<ShmidTable>,       // sysv进程共享内存表
    pub pending: AtomicBool,                   // 是否有信号待处理
    pub ucontext: AtomicUsize,                 // 信号用户态指针
    pub sig_pending: SpinNoIrqLock<SigPending>,// 信号列表
    pub blocked: SyncUnsafeCell<SigMask>,      // 任务阻塞信号
    pub handler: Shared<SigStruct>,            // 信号处理集合
    pub sig_stack: SyncUnsafeCell<Option<SignalStack>>, // 信号栈
    pub waker: SyncUnsafeCell<Option<Waker>>,  // 任务唤醒句柄
    pub trap_cx: SyncUnsafeCell<TrapContext>,  // 上下文
    pub time_data: SyncUnsafeCell<TimeData>,   // 时间戳
    pub cpuset: SyncUnsafeCell<CpuSet>,        // CPU亲和性掩码
    pub prio: SyncUnsafeCell<SchedParam>,      // 调度优先级和策略
    pub exit_code: AtomicI32,                  // 退出码
    /// CHILD_CLEARTID清除地址
    pub clear_child_tid: SyncUnsafeCell<Option<usize>>,
    /// CHILD_SETTID设置地址
    pub set_child_tid: SyncUnsafeCell<Option<usize>>,
    /// 父进程
    pub parent: Shared<Option<Weak<TaskControlBlock>>>,
    /// 子进程
    pub children: Shared<BTreeMap<usize, Arc<TaskControlBlock>>>,
}
```,
    caption: [任务控制块结构体],
    label-name: "task-control-block",
)

#h(2em)利用Rust Arc引用计数和clone机制，可以有效的解决进程和线程之间资源共享和隔离问题。对于可以共享的资源，调用 `Arc::clone()` 仅增加引用计数（原子操作），未复制底层数据，父子进程共享同一份数据。如果是可以独立的资源，调用clone会递归复制整个结构，生成完全独立的数据副本，父子进程修改互不影响。

=== 进程和线程联系

在 Del0n1x 中，我们使用 `ThreadGroup`管理线程组。
其中选择 `BTreeMap` 作为管理数据结构，其中key为task pid，value为TCB的弱引用。
线程之间需要一个leader，而该leader是一个进程，同样被 `ThreadGroup` 管理。
线程组的leader可以用tgid表示；利用tgid可以通过 `BTreeMap` 快速定位到leader。
线程与线程之间并没有父子关系，他们同属于一个进程创建。

#table(
  columns: (auto, auto, auto),
  align: (left, left, left),
  [*Type*], [*Characteristic*], [*Implementation*],
  [Process Leader], [tgid == pid],
  [
    新的内存空间，独立的信号处理程序
  ],
  [Thread], [tgid != pid],
  [
    通过 CLONE_VM 共享内存空间，通过 CLONE_SIGHAND 共享信号处理程序
  ]
)

#h(2em)进程和进程之间是树状结构，通过 `parent` 和 `children` 字段指明父进程和子进程。
Del0n1x 使用`Manager`管理任务和进程组，结构设计如下：

#code-figure(
```rs
pub struct Manager {
    pub task_manager: SpinNoIrqLock<TaskManager>,
    pub process_group: SpinNoIrqLock<ProcessGroupManager>,
}
/// 存放所有任务的管理器，可以通过pid快速找到对应的Task
pub struct TaskManager(pub HashMap<Pid, Weak<TaskControlBlock>>);
/// 存放进程组的管理器，通过进程组的leader 的pid可以定位到进程组
pub struct ProcessGroupManager(HashMap<PGid, Vec<Pid>>);
```,
    caption: [任务与进程组管理结构体],
    label-name: "manager-struct",
)

=== 任务的状态

在rCore的基础上，我们为任务在运行过程中设计了4种状态:

#list(
    [Ready: 任务已准备好执行，等待调度器分配CPU时间片;],
    [Running: 任务正在CPU上执行指令;],
    [Stopped: 任务被暂停执行，但未被终止，收到 SIGSTOP 信号],
    [Zombie: 任务已终止，但尚未被父进程回收],
    indent: 4em
)
// - Ready: 任务已准备好执行，等待调度器分配CPU时间片;
// - Running: 任务正在CPU上执行指令;
// - Stopped: 任务被暂停执行，但未被终止，收到 SIGSTOP 信号
// - Zombie: 任务已终止，但尚未被父进程回收

#h(2em)进程间状态转化如下：

#figure(
  image("assets/status.png"),
  caption: [进程状态转化图],
  supplement: [图],
)<进程状态转化>

#pagebreak()  // 强制分页