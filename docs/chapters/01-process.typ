#import "../template.typ": img, tbl, code-figure

= 进程管理

== 概述

Del0n1x操作系统采用无栈协程作为核心任务管理模型，该设计基于用户级轻量级线程理念，允许任务在执行过程中挂起并恢复。
无栈协程与有栈协程的核心差异体现在上下文管理机制上：在rcore和xv6中，采用有栈方式进行任务切换，每次任务切换需要调用`__switch`函数，每个任务有独立栈空间
并在切换时保存完整栈帧及`s0-s11`、`ra`、`sp`等14个RISC-V寄存器，
而Del0n1x的无栈协程不依赖独立栈结构，转而通过状态机管理上下文状态，任务挂起时仅需保存当前执行位置和关键状态变量，显著降低内存开销与切换延迟。

在传统操作系统中，进程调度需切换用户级上下文、寄存器上下文和系统级上下文（含内核栈与页表），这种完整上下文切换必须通过内核态系统调用完成。
但是Del0n1x采用共享内核栈架构，所有任务复用同一内核栈空间，任务切换时不涉及内核栈切换，这极大地降低了任务切换的开销。


== 任务调度

在无栈协程中，任务调度并不需要在栈上保存任务栈帧，而是将必要的中间信息保存为状态机，放置在堆空间进行保存，这样不仅减少了任务调度开销，同时
提高了调度过程中的安全性，降低栈溢出的风险。

在Del0n1x中，当对异步函数调用`.await`方法时，async-task库会首次调用Future的poll方法，如果poll返回的结果是Pending，那么该任务将被挂起，
await 将控制权交给调度器，以便另一个任务可以继续进行。任务调度如下图：

#figure(
    image("../assets/sched.png", width: 100%),
)<leaderboard>

=== 异步并不是"银弹"

我们在第一次接触异步调度时觉得，既然异步调度效率高，那为何不把所有的系统调用设计为异步形式呢？后来在深入了解到异步机制后发现，我们陷入了一个常见误区：把异步当成万能银弹。

首先异步的核心价值在于解决 I/O 瓶颈。 I/O 的延迟远高于 CPU 计算，磁盘/网络操作耗时可能是微秒（μs）甚至毫秒（ms）级，而 CPU 指令是纳秒（ns）级。异步 I/O 允许 CPU 在等待慢速 I/O 时执行其他任务。CPU 密集型操作无等待需求，如getpid(), sched_yield() 等系统调用本身不阻塞，异步化不会带来收益，反而增加调度开销。若一个操作本身能在短时间内完成，同步调用直接返回结果的开销远小于异步回调的上下文切换。

其次状态管理困难。异步操作需通过回调和细致的 Future 设计，如果想要精细的控制poll中代码处理流程，需要自己实现 Future 的 poll 轮循进行状态管理，这无疑会复杂化内核实现与bug调试。

== 任务调度队列与执行器

在任务调度队列实现中，Del0n1x将调度队列分为 FIFO 和 PRIO 队列，感谢优秀作品 Plntry 对 async-task 库做出的 Pr，
使得在任务调度过程中可以获取调度信息，通过调度信息，我们可以对任务做出更加精细的控制。
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

在Del0n1x中，我们使用统一的 TaskFuture 封装了任务。对于用户任务，在poll轮循中实现了任务的切换调度。当任务checkin时，需要在修改TCB的时间戳记录调度时间，然后切换CPU中运行任务和切换页表。当任务checkout时，需要判断浮点寄存器状态是否为dirty以确定是否保存浮点寄存器，然后清空CPU当前任务，并记录任务checkout时间。 对于内核任务，Del0n1x并没有设计任务切换，而是让该任务一直poll，直到任务结束，这类任务主要是shell程序。

#code-figure(
```rs
pub enum TaskFuture<F: Future<Output = ()> + Send + 'static> {
    UserTaskFuture {
        task: Arc<TaskControlBlock>,
        future: F,
    },
    KernelTaskFuture {
        future: F,
    },
}

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
    caption: [任务 Future 结构与 poll 实现],
    label-name: "task-future-poll",
)

spawn_user_task可以设置一个用户任务。Del0n1x将用户任务的future设置为 trap_loop 循环，负责处理任务在用户态和内核态之间的切换，直到任务结束。执行 `executor::spawn(future)` 将任务挂入全局队列中等待被调度。

#code-figure(
```rs
pub fn spawn_user_task(user_task: Arc<TaskControlBlock>) {
    let future = TaskFuture::user_task(user_task.clone(), trap_loop(user_task));
    executor::spawn(future);
}
```,
    caption: [用户任务生成函数],
    label-name: "spawn-user-task",
)

== 多核心CPU管理

在Del0n1x中，我们将处理器抽象为CPU，使用内核中CPU结构体进行统一管理。current 中保存当前正在运行的任务的 TCB；timer_irq_cnt 记录内核时钟中断次数，在内核时钟中断处理函数中会增加这个计数器，trap return时会清零，如果计数器大于阈值，手动对该任务yield进行调度，避免任务一直占用CPU；kernel_trap_ret_value 用于记录 pagafault 返回值。

#code-figure(
```rs
pub struct CPU {
    current: Option<Arc<TaskControlBlock>>,
    timer_irq_cnt: usize,
    hart_id: usize,
    kernel_trap_ret_value: Option<SysResult<()>>,
}
```,
    caption: [CPU 结构体定义],
    label-name: "cpu-struct",
)

单个CPU被存放在全局的 PROCESSORS 管理器中，并向外暴露接口，通过管理器我们能获取到当前任务的上下文信息和页表token、CPU id号等。

#code-figure(
```rs
const PROCESSOR: CPU = CPU::new();
pub static PROCESSORS: SyncProcessors = SyncProcessors(UnsafeCell::new([PROCESSOR; HART_NUM]));
```,
    caption: [全局 CPU 管理器],
    label-name: "processors-global",
)

== 任务控制块

进程是操作系统中资源管理的基本单位，而线程是操作系统中调度的基本单位。由于在linux设计理念中，线程是轻量级进程，所以在Del0n1x中使用统一的任务控制块来管理进程和线程。

#code-figure(
```rs
pub struct TaskControlBlock {
    pub pid: Pid,                                                 // 任务标识符
    pub tgid: AtomicUsize,                                        // leader的pid号
    pub pgid: AtomicUsize,                                        // 进程组id
    pub task_status: SpinNoIrqLock<TaskStatus>,                   // 任务状态
    pub thread_group: Shared<ThreadGroup>,                        // 线程组
    pub memory_space: Shared<MemorySpace>,                        // 地址空间
    pub parent: Shared<Option<Weak<TaskControlBlock>>>,           // 父进程
    pub children: Shared<BTreeMap<usize, Arc<TaskControlBlock>>>, // 子进程
    pub fd_table: Shared<FdTable>,                                // 文件描述表
    pub current_path: Shared<String>,                             // 路径
    pub robust_list: Shared<RobustList>,                          // 存储线程的信息
    pub futex_list: Shared<FutexBucket>,                          // futex互斥锁队列
    pub itimers: Shared<[ITimerVal; 3]>,                          // 任务的内部时钟
    pub fsz_limit: Shared<Option<RLimit64>>,                      // 任务的资源限制
    pub shmid_table: Shared<ShmidTable>,
    pub pending: AtomicBool,                            // 是否有信号待处理
    pub ucontext: AtomicUsize,                          // 信号用户态指针
    pub sig_pending: SpinNoIrqLock<SigPending>,         // 信号列表
    pub blocked: SyncUnsafeCell<SigMask>,               // 任务阻塞信号
    pub handler: Shared<SigStruct>,                     // 信号处理集合
    pub sig_stack: SyncUnsafeCell<Option<SignalStack>>, // 信号栈
    pub waker: SyncUnsafeCell<Option<Waker>>,           // 任务唤醒句柄
    pub trap_cx: SyncUnsafeCell<TrapContext>,           // 上下文
    pub time_data: SyncUnsafeCell<TimeData>,            // 时间戳
    pub clear_child_tid: SyncUnsafeCell<Option<usize>>, // CHILD_CLEARTID清除地址
    pub set_child_tid: SyncUnsafeCell<Option<usize>>,   // CHILD_SETTID设置地址
    pub cpuset: SyncUnsafeCell<CpuSet>,                 // CPU亲和性掩码
    pub prio: SyncUnsafeCell<SchedParam>,               // 调度优先级和策略
    pub exit_code: AtomicI32,                           // 退出码
}
```,
    caption: [任务控制块结构体],
    label-name: "task-control-block",
)

利用rust Arc引用计数和clone机制，可以有效的解决进程和线程之间资源共享和隔离问题。对于可以共享的资源，调用 Arc::clone() 仅增加引用计数（原子操作），未复制底层数据，父子进程共享同一份数据。如果是可以独立的资源（如memory_space），调用clone会递归复制整个结构，生成完全独立的数据副本，父子进程修改互不影响。

=== 进程和线程联系

在 Del0n1x 中，我们使用 `ThreadGroup`管理线程组。
其中选择 `BTreeMap` 作为管理数据结构，其中key是task pid，value是TCB的弱引用。
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

进程和进程之间是树状结构，通过 parent 和 children 字段指明父进程和子进程。
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

在rcore的基础上，我们为任务在运行过程中设计了4种状态:

- Ready: 任务已准备好执行，等待调度器分配CPU时间片;
- Running: 任务正在CPU上执行指令;
- Stopped: 任务被暂停执行，但未被终止，收到 SIGSTOP 信号
- Zombie: 任务已终止，但尚未被父进程回收

#code-figure(
```rust
pub enum TaskStatus {
    Ready,
    Running,
    Stopped,
    Zombie,
}
```,
    caption: [任务状态枚举],
    label-name: "task-status-enum",
)

#figure(
    image("../assets/status.png", width: 100%),
)<leaderboard>