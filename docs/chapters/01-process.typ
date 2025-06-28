#import "../template.typ": img, tbl

= 进程管理

== 任务调度

=== 为什么选择无栈协程

协程是一种比线程更加轻量级的并发单位，允许在执行过程中挂起并稍后恢复，从而使得单个线程可以处理多个任务。无栈协程和有栈协程的主要区别在于它们的上下文管理方式。

- #strong[有栈协程（Stackful
  Coroutine）]：每个协程都有自己的栈，这样可以在协程内部随时进行函数调用和上下文切换，在大规模并发时会带来较大的内存开销。在 RISC-V 架构的操作系统中，有栈协程需要在栈中保存`s0-s11`、`ra`、`sp`这 14 个寄存器。
- #strong[无栈协程（Stackless
  Coroutine）]：协程没有独立的栈，而是依赖于状态机来管理上下文，每次协程挂起时需要保存执行的状态和位置，下次恢复时从该位置继续执行。无栈协程非常轻量，适合大规模并发。但是不适合深度嵌套的函数调用，适合用在状态转移和事件驱动的场景中。

在操作系统中，每一个线程都有其上下文环境，当发生任务调度时，任务切换其实就是上下文切换。任务是由内核来管理和调度的，任务的切换必须发生在内核态。

在 Linux 系统中，一个进程的上下文可以分为以下三个部分：

+ 用户级上下文：包括了进程在用户态下的运行状态和资源。如用户堆栈、全局变量和静态变量等等
+ 寄存器上下文：如通用寄存器、栈指针
+ 系统级上下文：如进程控制块、内核栈、页表

当 Linux 发生进程调度时，必须对上面的全部信息进行切换。通过调用函数的方法 (Linux 内核中 switch\_to 函数) 进行上下文切换。


但是，Phoenix 使用无栈协程架构，所有任务共享同一个内核栈，任务调度时不需要切换内核栈。Phoenix 的任务切换发生在`await`处，当`poll`轮询返回`pending`时不断退出本层`async`函数返回到上一层的`async`函数，直到回到调度器，接着调度器调度到下一个任务，并根据堆上的信息重新生成新任务的函数调用栈，来到上次该任务 `await` 的地方继续`poll`轮询。

我们选择无栈协程，除了其任务切换开销小以外，很大一部分原因也与 Rust 语言本身的异步编程模型有关。

Rust 的所有权系统通过编译时检查保证内存安全，防止数据竞争、空指针和悬挂指针等问题。Rust 的所有权和借用检查，能够更好地保证协程在并发执行时的安全性，确保资源在使用后会被正确释放，更加高效地利用系统资源。

另外，Rust 内置的异步编程模型通过`async`和`await`关键字支持无栈协程，这种语法糖使得编写和使用无栈协程变得更加简洁和直观。每个`async`函数在编译时会被转换为状态机，自动管理状态的保存和恢复。无栈协程符合 Rust 所追求的“零成本抽象”，状态机的转换和上下文切换在编译期确定，运行时开销极小。在敏捷开发过程中，这种语法糖能极大提高操作系统的开发效率。

此外，Rust 活跃的社区和丰富的生态系统也提供了大量用于异步编程的库和工具。Phoenix 在无标准库`no-std`的环境下开发，因此可以利用`async-task`库提供的关于 `Future`的抽象实现自己的全局任务队列调度器。

=== Rust 无栈协程原理

==== 将异步函数编译成状态机

2019 年底 Rust 推出了支持异步编程的`async` 和 `await`
关键字，极大地简化了异步函数的定义和调用，并在无栈协程调度中发挥了重要作用。

- `async` 关键字用于定义异步函数、异步块或异步闭包。当使用 `async`
  标记一个函数时，该函数不再同步返回结果，而是返回一个实现了 `Future`
  trait 的对象。在编译时，带有 `async`
  关键字的代码会被转换为一个状态机。这个状态机会在每个 `await`
  点保存当前状态，并在未来的某个时刻恢复执行。
- `await` 关键字用于等待一个实现了 `Future` trait
  的对象完成，并获取其结果。当执行到 `await` 时，如果 `Future`
  尚未完成，当前任务会挂起，并允许其它任务继续执行。每当 `Future`
  完成时，使用 `await`
  的代码会从挂起点恢复执行，状态机会从之前保存的状态继续运行。

下面这个例子演示了这两个关键字的使用：

```rust
async fn async_function() {
    println!("First part of the function");
    async_operation().await;
    println!("Second part of the function");
}

async fn async_operation() {
    // 模拟异步操作
    println!("Performing async operation");
}
```

Rust 编译器会将上面这段代码编译成一个实现`Future`
trait 的状态机。这个状态机记录了函数执行的当前位置和需要恢复的状态：

```rust
enum AsyncState {
    FirstPart,
    AwaitingOperation,
    SecondPart,
    Done,
}

struct AsyncFunction {
    state: AsyncState,
}

impl AsyncFunction {
    fn new() -> Self {
        AsyncFunction {
            state: AsyncState::FirstPart,
        }
    }
}

impl Future for AsyncFunction {
    type Output = ();
    fn poll(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            match self.state {
                AsyncState::FirstPart => {
                    println!("First part of the function");
                    self.state = AsyncState::AwaitingOperation;
                }
                AsyncState::AwaitingOperation => {
                    println!("Performing async operation");
                    self.state = AsyncState::SecondPart;
                    return Poll::Pending;
                }
                AsyncState::SecondPart => {
                    println!("Second part of the function");
                    self.state = AsyncState::Done;
                }
                AsyncState::Done => {
                    return Poll::Ready(());
                }
            }
        }
    }
}
```

上面这段代码涉及到如下几个关键的概念：

- #strong[`Future` Trait]：我们可以将`Future`
  理解为一个异步计算的抽象，它的 `poll` 方法会尝试推进计算。`Future`
  trait
  类似于一个任务的描述，告诉你这个任务将在未来完成，并且可以查询其状态。

- #strong[`Poll` 枚举]：表示 `Future`
  的当前状态（注意`Poll`枚举不是`poll`方法，两者是两个概念）：

  - `Poll::Pending`：异步操作尚未完成，需要等待。

  - `Poll::Ready(T)`：异步操作已经完成，可以返回结果。

- #strong[`Pin`]：`poll` 方法的 `self` 参数被包裹在 `Pin<&mut Self>`
  中，确保 `AsyncFunction` 的内存位置固定，不会被移动，从而保证安全。

- #strong[`Context`] :包含了执行 `Future` 所需的上下文信息，例如
  `Waker`，它允许 `Future` 在准备好继续执行时通知执行器。`Context` 使得
  `Future` 可以与外部世界交互，知道何时需要继续执行。

==== Executor 和 Reactor

// TODO: Reactor 的介绍有可能不对

Executor（执行器）是一个运行时组件，它负责管理和调度异步任务。Executor 的主要职责包括：

+ #strong[运行 Futures]：Executor 会调用 Futures 的`poll`方法来推进它们的执行。
+ #strong[任务队列]：Executor 维护一个任务队列，存放所有准备好执行的任务。
+ #strong[循环调度]：Executor 在循环中轮询任务队列，逐个执行任务。当任务返回`Poll::Ready`时，表示任务已经完成；当返回`Poll::Pending`时，表示任务尚未完成，需要等待某些条件。

Reactor（反应器）是另一个运行时组件，负责管理和处理异步 I/O 操作或其他需要等待的事件。Reactor 的主要职责包括：

+ #strong[等待事件]：Reactor 等待异步操作的完成，例如网络 I/O、文件 I/O、计时器等。
+ #strong[唤醒任务]：当异步操作完成时，Reactor 会唤醒相应的任务，并将它们重新放入 Executor 的任务队列，以便再次执行。

=== 任务调度队列与执行器

得益于 `async_task` 对 `Future` 提供的便捷的抽象，Phoenix 自己实现了全局任务队列调度器。

为了存储待执行的协程任务，Phoenix 定义了一个全局的任务队列`TASK_QUEUE`，各个 CPU 可以从该任务队列中获取任务执行。`TaskQueue`是一个使用 `SpinNoIrqLock` 保护一个双端队列（`VecDeque`），`Runnable` 为可调度的任务：

```rust
struct TaskQueue {
    queue: SpinNoIrqLock<VecDeque<Runnable>>,
}
```

`spawn` 函数用于创建并启动一个新的异步任务。它接收一个实现了 `Future` 的对象，并将其转换为一个可以调度和执行的任务并添加到任务队列中。

```rust
pub fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let schedule = move |runnable: Runnable, info: ScheduleInfo| {
        if info.woken_while_running {
            // i.e `yield_now()`
            TASK_QUEUE.push(runnable);
        } else {
            // i.e. woken up by some signal
            TASK_QUEUE.push_preempt(runnable);
        }
    };
    async_task::spawn(future, WithInfo(schedule))
}
```

任务的调度策略如下：

- 如果任务在运行时被唤醒（`woken_while_running`），将任务放到队列尾部，按照先入先出的顺序执行。这种情况通常发生在任务调用`yield_now()`主动让出处理器时。
- 如果任务在睡眠中被唤醒，将任务放到队列头部，以确保优先执行。这种情况通常发生在任务已经休眠正在等待某个事件，此时等待的事件发生时将该任务唤醒。也可能发生在任务在处于`Interruptable`状态，此时收到了一个未阻塞的信号并且触发了 `sighandler` 函数，将任务唤醒。

`run_until_idle`
函数负责从任务队列中取出并执行任务，直到队列为空。Phoenix 支持多核并发，每一个 CPU 核心都在一个永不停止的 loop 循环中持续调用该函数

```rust
pub fn run_until_idle() {
    while let Some(task) = TASK_QUEUE.fetch() {
        task.run();
    }
}
```

=== 异步任务上下文切换

每一个任务都有独立的上下文，在进行上下文切换时，Phoenix 需要切换任务控制块、`sum_cnt` 计数器（用来维护内核态对用户内存的访问限制）、页表。

在 RISC-V 中，`SUM` 标志位的作用是控制在内核态（特权级 S 模式）下，是否允许内核访问用户态（U 模式）的内存。Phoenix 在内核态处理系统调用访问用户态指针时需要维护 `sum_cnt` 计数器，当计数器大于 0，`SUM` 标志位为 1，此时内核可以安全地读取或写入用户态缓冲区。内核完成对用户态缓冲区的操作后，`sum_cnt` 计数器减一。当减到 0 时需要恢复 `SUM` 位为 0 以避免意外访问用户态内存。

实际上，上下文的切换可以与 Rust 异步无栈协程完美结合。在每一个用户异步任务的最外层都套了一层`UserTaskFuture`:

```rust
pub struct UserTaskFuture<F: Future + Send + 'static> {
    task: Arc<Task>,
    env: EnvContext,
    future: F,
}

impl<F: Future + Send + 'static> Future for UserTaskFuture<F> {
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let hart = hart::local_hart();
        hart.enter_user_task_switch(&mut this.task, &mut this.env);
        let ret = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };
        hart.leave_user_task_switch(&mut this.env);
        ret
    }
}
```

`UserTaskFuture`
的`EnvContext`字段存放了 `sum_cnt` ，`task`字段存储了任务控制块，页表保存在任务控制块内部。`enter_user_task_switch`函数将当前 CPU 的上下文切换为即将执行的`task`，执行完毕后，`leave_user_task_switch`则将页表切换回内核页表，任务调度器将会轮询下一个 `UserTaskFuture`。注意切换任务的时候需要关中断，否则可能产生死锁。

`UserTaskFuture`中的`future`字段对于每一个任务而言都是同一个异步函数`task_loop`，也就是在`task`的生命周期内不断进行返回用户态、陷入内核态、信号处理等过程，直到`task`结束，状态变为`Zombie`后退出循环，交由`do_exit`函数进行处理。

== 任务控制块

进程是操作系统中资源分配的基本单位。每个进程都有自己独立的地址空间和资源，如内存、文件描述符等。线程是操作系统中 CPU 调度的基本单位。线程共享所在进程的地址空间和资源，但有独立的执行上下文。

在调研其他队的设计时我们发现，很多队都将进程和线程分开设计，分别使用 Process 和 Thread 结构体表示。但其实在 Linux 内核中，并没有严格地区分进程和线程，而是通过一组统一的 API 来操作任务，例如`sys_clone`系统调用通过不同的 `flags` 组合创建共享不同资源的新任务，因此进程和线程的创建本质上是类似的。统一使用 `Task` 结构体，可以通过标志位决定任务的具体属性，而不需要区分进程结构体和线程结构体。比如，设置 `CLONE_VM` 标志位时，新任务共享父任务的地址空间，这可以通过 `Task` 结构体中的 `memory_space` 字段来实现共享。同样，设置 `CLONE_THREAD` 标志位时，新任务加入父任务的线程组，这可以通过 `Task` 结构体中的 `ThreadGroup` 字段来实现管理。

=== Task 结构体设计

基于上述分析，我们对进程和线程统一使用`Task`结构体表示，其具体设计如下：

```rust
/// User task control block, a.k.a. process control block.
///
/// We treat processes and threads as tasks, consistent with the approach
/// adopted by Linux. A process is a task that is the leader of a
/// `ThreadGroup`.
pub struct Task {
    // Immutable
    /// Task identifier handle.
    tid: TidHandle,
    /// Weak reference to the leader task. `None` if this task is the leader.
    leader: Option<Weak<Task>>,
    /// Indicates if the task is the leader of its thread group.
    is_leader: bool,

    // Mutable
    /// Indicates if the task is a zombie. Protected by a spin lock due to
    /// potential access by other tasks.
    state: SpinNoIrqLock<TaskState>,
    /// The address space of the process.
    memory_space: Shared<MemorySpace>,
    /// Map of start address of shared memory areas to their keys in the
    /// shared memory manager.
    shm_ids: Shared<BTreeMap<VirtAddr, usize>>,
    /// Parent process.
    parent: Shared<Option<Weak<Task>>>,
    /// Children processes.
    children: Shared<BTreeMap<Tid, Arc<Task>>>,
    /// Exit code of the current process.
    exit_code: AtomicI32,
    /// Trap context for the task.
    trap_context: SyncUnsafeCell<TrapContext>,
    /// Waker to add the task back to the scheduler.
    waker: SyncUnsafeCell<Option<Waker>>,
    /// Thread group containing this task.
    thread_group: Shared<ThreadGroup>,
    /// File descriptor table.
    fd_table: Shared<FdTable>,
    /// Current working directory dentry.
    cwd: Shared<Arc<dyn Dentry>>,
    /// Pending signals for the task.
    sig_pending: SpinNoIrqLock<SigPending>,
    /// Signal handlers.
    sig_handlers: Shared<SigHandlers>,
    /// Signal mask for the task.
    sig_mask: SyncUnsafeCell<SigSet>,
    /// Optional signal stack for the task, settable via `sys_signalstack`.
    sig_stack: SyncUnsafeCell<Option<SignalStack>>,
    /// Pointer to the user context for signal handling.
    sig_ucontext_ptr: AtomicUsize,
    /// Statistics for task execution times.
    time_stat: SyncUnsafeCell<TaskTimeStat>,
    /// Interval timers for the task.
    itimers: Shared<[ITimer; 3]>,
    /// Futexes used by the task.
    robust: Shared<RobustListHead>,
    /// Address of the task's thread ID.
    tid_address: SyncUnsafeCell<TidAddress>,
    /// Mask of CPUs allowed for the task.
    cpus_allowed: SyncUnsafeCell<CpuMask>,
    /// Process group ID of the task.
    pgid: Shared<PGid>,
    /// ELF file the task executes.
    elf: SyncUnsafeCell<Arc<dyn File>>,
    /// Command-line arguments for the task.
    args: SyncUnsafeCell<Vec<String>>,
}
```

下面介绍一下 `Task` 结构体各个字段的含义。

首先 `Task` 结构体被大致划分为不可变（Immutable）和可变（Mutable）两部分：

- #strong[不可变部分]：不可变字段在任务创建后不会改变，因此可以保证数据的一致性。这些字段通常包括任务的基本标识信息，如
  `tid`（任务 ID）和
  `is_leader`（是否为线程组的主线程）。由于这些字段不会改变，多线程环境下访问这些字段时不需要加锁，可以提高访问效率和安全性。

#align(center)[#table(
  columns: 2,
  align: horizon,
  inset: 10pt,
  [字段名], [含义],
  [`tid`],
  [任务的唯一标识符（任务 ID）],
  [`leader`],
  [对主线程任务的弱引用。如果该任务是线程组的主线程，则为 `None`],
  [`is_leader`],
  [标识任务是否是线程组的主线程],
)
]

- #strong[可变部分]：可变字段涉及任务的状态和资源管理，如
  `state`（任务状态）、`memory_space`（内存空间）和
  `children`（子任务）。这些字段在任务的生命周期内可能会改变，需要使用锁机制来保证线程安全。


对于 `Task` 的可变部分，我们也进行了巧妙设计，调研其他队时我们发现，很多队都用了`Inner`结构体存储可变字段，并在外面加上自旋锁的方式，此种方式虽然能在并发时保证数据的安全性，但是用一把大锁来锁定可变字段并不高效。因此，我们根据可变字段是线程内部独占还是在线程之间共享，分别使用`SyncUnsafeCell`和`Shared<T>`(即`Arc<SpinNoIrqLock<T>>`) 对可变字段进行包裹，并通过谨慎使用，保证了并发安全性：

- `SyncUnsafeCell<T>`：对于线程独有的字段使用 `SyncUnsafeCell` 包裹。`SyncUnsafeCell` 是一种可以在线程间共享的 `UnsafeCell`。虽然 `UnsafeCell` 本身不实现 `Sync`，`SyncUnsafeCell` 可以在 `T` 实现 `Sync` 的情况下实现 `Sync`。这使得它可以在多线程环境中共享，并且仍然允许内部可变性。

- `Arc<SpinNoIrqLock<T>>`：对同一进程内线程共享的字段使用。`SpinNoIrqLock` 是一种自旋锁，适用于短时间的锁定操作。它不会引发中断，因此适用于操作系统内核中需要快速响应的部分。使用 `SpinNoIrqLock` 包裹的字段可以在并发访问时进行保护，避免数据竞争和不一致。`Arc`（原子引用计数）允许多个所有者共享同一个数据，同时维护共享数据的引用计数，以确保数据在最后一个引用被丢弃时才被释放。

在 `Task` 结构体中使用 `Shared<T>` 和 `SyncUnsafeCell<T>`
包裹可变字段，可以有效地实现内部可变性和细粒度锁定。这种设计允许在多个任务之间安全地共享和修改数据，提高了系统的并发性和性能，同时保证了线程安全性。通过这些机制，操作系统能够更加高效地管理任务和资源。

=== `ThreadGroup`与进程线程的关系

// TODO：进程和线程组的关系没讲明白，应该讲些主线程，比如 Linux 怎么处理

在 Linux 中，线程和进程的关系比一些传统操作系统更加紧密。Linux 使用 `task_struct` 数据结构来表示线程和进程，线程和进程在内核中的表示实际上是一样的。一个线程或进程都是一个 `task_struct` 实例。进程实际上被抽象成了线程组，其主要由线程组的主线程进行管理。线程组是由同一个进程创建的所有线程组成的组。每个线程组有一个主线程（通常是创建线程的进程），其 `TID` (线程标示符) 是线程组的 `TGID`（线程组标识符）。从系统角度来看，线程组的主线程表示整个进程，而每个线程的 `TID` 是其在进程中的标识。

Phoenix 效仿 Linux，对进程和线程用`Task`结构体进行统一表示，`ThreadGroup`
结构体用于管理属于同一进程的线程组。通过将多个线程归属于一个
`ThreadGroup`，可以方便地管理和调度同一进程中的所有线程。`Task` 结构体和
`ThreadGroup` 结构体的结合，实现了统一的进程和线程管理。

```rust
pub struct ThreadGroup {
    members: BTreeMap<Tid, Weak<Task>>,
}
```

`ThreadGroup` 结构体包含一个 `BTreeMap`，该映射将线程的唯一标识符
(`Tid`) 映射到对 `Task` 的弱引用
(`Weak<Task>`)。当新创建一个进程时，`ThreadGroup` 仅有进程本身这个成员，`Task` 的`is_leader`字段设为`True`。当`sys_clone`设置
`CLONE_THREAD`
标志位时，将新创建的`Task`添加入`ThreadGroup`，新`Task`的`is_leader`字段设为`False`。

=== 任务的状态

在调研其他操作系统时，我们发现部分往届作品如 Titanix 只是简单区分了`Running`和`Zombie`状态，并不支持当`SIGSTOP`信号到来时进程暂停执行，此外，任务在阻塞的过程中能否被信号打断也支持得并不是很好，Linux 关于信号的手册中有提到，部分系统调用在阻塞等待的过程中可以被信号打断停止执行，返回`EINTR`错误，如果打断系统调用的信号的`flag`标志中含有`SA_RESTART`可以重启系统调用。基于以上考虑，我们将 `Task` 的状态分为以下五种：

#align(center)[#table(
  columns: 2,
  align: (col, row) => (auto,auto,).at(col),
  inset: 10pt,
  [状态], [含义],
  [`Running`],
  [正在运行或准备运行的任务。此状态下，任务占用 CPU，执行其代码。],
  [`Zombie`],
  [任务已终止，但其进程控制块 (PCB)
  仍然存在，以便父进程可以读取其退出状态。],
  [`Stopped`],
  [任务已停止运行，通常是由于接收到停止信号（如
  `SIGSTOP`）。可以通过特定信号（如 `SIGCONT`）恢复运行。],
  [`Interruptable`],
  [任务处于可中断的等待状态，等待某个事件（如 I/O
  操作完成或资源释放）。此状态下，任务可以被信号中断并唤醒。],
  [`UnInterruptable`],
  [任务处于不可中断的等待状态，等待某个事件的发生。此状态下，任务不会被信号中断，以确保某些关键操作的完整性和原子性。],
)
]

进程间的状态转换情况如下：

+ #strong[Running #sym.arrow.l.r Interruptable]：当任务需要等待某个事件时，从
  `Running` 转换到 `Interruptable`
  状态。例如，在实现`sys_wait4`系统调用时，`Task` 调用`suspend_now()`将自己从任务调度队列中移除，进入`Interruptable`状态，即可以被信号中断，如果是等待的子进程退出，返回子进程的`pid`，如果是被信号中断，返回`EINTR`错误。之后从`Interruptable`状态恢复为`Running`
+ #strong[Running #sym.arrow.l.r UnInterruptable]：当任务需要等待某个关键事件且不希望被信号中断时，从 `Running` 转换到
    `UnInterruptable` 状态。当等待的事件发生时，恢复到 `Running` 状态
+ #strong[Running #sym.arrow.r Zombie]：当任务执行结束并退出时，从 `Running` 转换到 `Zombie` 状态。
+ #strong[Running #sym.arrow.r Stopped]：当任务接收到停止信号（如 `SIGSTOP`）时，从 `Running` 转换到 `Stopped` 状态。
+ #strong[Stopped #sym.arrow.r Running]：当任务接收到继续信号（如 `SIGCONT`）时，从 `Stopped` 转换回 `Running` 状态。

== 中断与异常

中断与异常是计算机系统中非常重要的机制，它们使得操作系统能够响应外部事件和处理各种异常情况。当中断或异常发生时，处理器会暂停当前正在执行的任务，保存其状态，并跳转到专门的中断处理程序来处理这些事件。这种机制确保了系统的稳定性和安全性。

在处理中断和异常时，操作系统需要在内核态和用户态之间切换。这种切换涉及到内存管理和页表的操作，因此页表的设计和使用变得尤为关键。

在 Linux 操作系统中，Linux 内核使用了 Page Table Isolation
(PTI) 机制，内核态和用户态的页表相互独立，当系统运行在用户态时，仅使用用户页表，这个页表只包含极少量必须的内核数据，如用于进入和退出内核的函数和中断描述符表（IDT）。当系统进入内核态（通过系统调用、中断或异常）时，切换到完整的内核页表。这种设计的主要优点包括：

+ #strong[提高安全性]：独立的页表可以有效防止侧信道攻击，使得用户态程序无法访问敏感的内核数据。
+ #strong[保护内核空间]：即使攻击者能够突破用户态的防线，也无法直接访问或利用内核地址空间。

然而，这种设计也带来了额外的开销：

+ #strong[内存使用增加]：每个进程需要维护两套页表，增加了内存消耗。
+ #strong[性能开销]：每次从用户态切换到内核态（或反之）时，都需要切换页表，这涉及到刷新 TLB（Translation
  Lookaside Buffer），增加了系统调用和中断处理的成本

Phoenix 采用了用户态与内核态共用一个页表的策略，共用一个页表避免了在用户态和内核态之间切换时刷新 TLB 的开销，从而提升了系统性能，并且简化了页表管理的实现和维护。

在 Phoenix 中，`Task` 的生命周期就是在执行下面这个函数：

```rust
pub async fn task_loop(task: Arc<Task>) {
    *task.waker() = Some(get_waker().await);
    loop {
        match task.state() {
            Terminated => break,
            Stopped => suspend_now().await,
            _ => {}
        }
        trap::user_trap::trap_return(&task);
        // task may be set to terminated by other task, e.g. execve will 
        // kill other tasks in the same thread group
        match task.state() {
            Terminated => break,
            Stopped => suspend_now().await,
            _ => {}
        }
        let intr = trap::user_trap::trap_handler(&task).await;
        match task.state() {
            Terminated => break,
            Stopped => suspend_now().await,
            _ => {}
        }
        do_signal(&task, intr).expect("do signal error");
    }
    task.do_exit();
}
```

即 `Task` 先创建，然后执行`trap_return`返回用户态、陷入内核态由`trap_handler`处理中断和异常、`do_signal`执行信号处理函数，当 Task 被设置为`Zombie`状态时，回收部分资源，在`Wait4`系统调用时彻底回收 `Task` 资源。

在 Phoenix 的内核态用户态切换中，`TrapContext`
是一个关键的数据结构，用于保存从用户态切换到内核态，以及从内核态切换回用户态时需要保存和恢复的上下文信息。这个结构体的设计保证了用户态和内核态之间的切换能够正确地进行，不会丢失任何重要的状态信息。

```rust
pub struct TrapContext {
    // NOTE:  User to kernel should save these:
    /// General regs from x0 to x31.
    pub user_x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus, // 32
    /// CSR sepc
    pub sepc: usize, // 33

    // NOTE: Kernel to user should save these:
    pub kernel_sp: usize, // 34
    pub kernel_ra: usize, // 35
    pub kernel_s: [usize; 12], // 36 - 47
    pub kernel_fp: usize, // 48
    pub kernel_tp: usize, // 49

    /// Float regs
    pub user_fx: UserFloatContext,
    /// used for the `SA_RESTART` flag of signal 
    pub last_a0: usize,
}
```

=== 内核态 #sym.arrow.r 用户态

Phoenix 中内核态返回到用户态需要执行 `trap_return` 函数：

// 在 Phoenix 中，从内核态切换到用户态发生在以下两种情况：

// TODO: 很怪

// + `Task` 从 ELF 文件中构建出来时
// + 因为内核态的系统调用、中断或异常处理完成重新返回用户态时

// 通常情况下，在返回用户态之前需要执行信号处理函数，不过在第一种情况中，`Task` 刚被创建时还没有收到信号，因此`loop`循环第一行是直接`trap_return`，将`do_signal`放在`loop`的结尾。

```rust
#[no_mangle]
pub fn trap_return(task: &Arc<Task>) {
    unsafe {
        disable_interrupt();
        set_user_trap()
    };
    task.time_stat().record_trap_return();

    // Restore the float regs if needed.
    // Two cases that may need to restore regs:
    // 1. This task has yielded after last trap
    // 2. This task encounter a signal handler
    task.trap_context_mut().user_fx.restore();
    task.trap_context_mut().sstatus.set_fs(FS::Clean);
    unsafe {
        __return_to_user(task.trap_context_mut());
        // NOTE: next time when user traps into kernel, it will come back 
        // here and return to `user_loop` function.
    }
    task.trap_context_mut()
        .user_fx
        .mark_save_if_needed(task.trap_context_mut().sstatus);
    task.time_stat().record_trap();
}

```

在返回用户态之前，为了保证上下文切换的原子性，先禁用中断以防止在切换过程中发生新的中断。调用`set_user_trap()`函数设置陷阱处理函数，以确保下一次从用户态陷入内核态时能够正确处理。使用`__return_to_user`汇编函数，将之前存储的用户寄存器恢复。

在返回用户态时，将`TrapContext`的地址存储在`sscratch`寄存器中，`a0`寄存器指向`TrapContext`。再保存内核态的寄存器，切换栈指针并恢复状态寄存器、通用寄存器、用户态指针，最后使用`sret`指令返回用户态。

=== 用户态 #sym.arrow.r 内核态

在 Phoenix 中，从用户态切换到内核态发生在以下三种情况：

+ #strong[系统调用]：用户程序请求内核服务。
+ #strong[中断]：硬件设备需要处理。
+ #strong[异常]：用户程序执行非法操作。

当从用户态陷入内核态时，会进入`__trap_from_user`标签定义的代码。这段汇编代码的作用是保存用户态的上下文，并准备好内核态的环境。接着进入`trap_handler`函数根据不同的陷阱类型进行处理。

`trap_handler`函数会获取任务的陷阱上下文、读取`stval`、`scause`、`sepc`寄存器的值，得到引起陷阱的虚拟地址、引起陷阱的原因、引起陷阱的指令地址。如果是系统调用则依据无栈协程架构异步执行系统调用，如果是页面错误则调用`handle_page_fault`函数，如果是非法指令则终止任务，如果是定时器中断则让出处理器等待下次调度。

=== 内核态 #sym.arrow.r 内核态

在 MIT 的 xv6 操作系统与清华大学的 rCore-Tutorial 操作系统中，并不支持内核态的中断，这会导致内核对部分中断的响应不及时，在 rCore-Tutorial 操作系统中甚至会直接 Panic。

Phoenix 允许嵌套中断，例如内核态在收到时钟中断时，或者在执行系统调用中遇到访存异常时，依然会对中断进行处理。

内核态发生中断时，Phoenix 依次执行如下操作：

- #strong[保存调用者保存寄存器]：在陷阱发生时，保存调用者保存的寄存器（`ra`,
  `t0-t6`, `a0-a7`）。
- #strong[调用陷阱处理函数]：调用 `kernel_trap_handler` 处理陷阱。
- #strong[恢复调用者保存寄存器]：在陷阱处理完成后，恢复之前保存的寄存器。
- #strong[返回指令]：使用 `sret` 指令返回到陷阱发生前的执行点。

此外，在执行系统调用时会涉及到用户态指针的读写，Phoenix 采取的是直接解引用用户态指针的方法提高系统调用的执行速度。但是用户态传入的指针可能指向了非法内存，此时对非法内存的访问也会造成内核态的中断。为此，Phoenix 在读写用户态指针时，将中断处理函数单独设置为`__user_rw_trap_vector`对内存读写产生的中断进行单独处理。


== 内核态抢占式调度

以往采用无栈协程架构的作品都有一个共同的缺陷，那就是不支持内核态抢占式调度。首先，什么是抢占式调度？抢占式调度是指内核在时钟中断到来时能够强制切换当前正在运行的任务。然而，对于无栈协程来说，协程本身不支持抢占式调度，只能在 async 代码块中调用异步 `yield_now` 函数的方式主动让出当前协程。这是因为无栈协程需要在编译期被编译成状态机，因此调度的时刻需要在编译期确定，也就是说必须在异步环境中进行。因此，它不能像传统有栈协程那样在任何时刻发生中断后，在同步中断处理函数中直接切换任务和任务栈。


尽管如此，无栈协程可以支持用户态抢占。因为用户态线程在中断后会陷入内核态，内核态可以使用异步函数 `task_loop` 去捕获用户态的所有中断和陷入，因此可以在时钟中断时调用 `yield_now` 函数主动让出当前协程。然而，在内核态中断发生时，内核本身的异步环境被打破了，因此内核中断处理函数必须是同步的，原则上不能发生调度。这也是为什么以往所有无栈协程作品都没有支持内核态抢占式调度的原因。这是无栈协程相对于有栈协程来说的固有缺陷，不支持抢占式调度。

Phoenix 打破了这一限制，在无栈协程的基础上创新性地发展出了内核态抢占式调度的方式。具体来说，在每个内核态时钟中断发生时，内核必定会陷入同步的中断处理函数中，在同步函数中不能调度，那么Phoenix就在同步函数中，在当前栈的基础上运行一个新的协程。为了更好理解，拿自旋关中断锁来作比喻，自旋锁不是已经保持互斥访问了吗，为什么还要用自旋关中断锁呢？因为如果使用自旋锁，内核可能在上锁期间陷入内核态中断，这时如果内核中断处理函数访问同一个锁，就会导致死锁的情况。因此我们需要在上锁前需要关中断。这个例子说明了单核情况也会发生的锁争用问题，但也暗含了内核态陷入中断时的并发特征，内核态中断时可以看作是原来正在执行的CPU暂停，一个新的CPU加入。这也是Phoenix面对内核态抢占式调度的解决方法，我们在内核态中断时，将当前正在执行的协程当作是暂停状态，然后在它的栈的基础上继续运行新的协程，如果新的协程发生调度就退出内核中断处理函数，回到原来被抢占的协程继续执行，这就是无栈协程在内核态的抢占式调度。内核栈的变化如@kernel-trap-preempt 所示。

#img(
   image("../assets/kernel-trap-preempt.png"),
   caption: "内核态抢占式调度前后栈变化"
)<kernel-trap-preempt>


为了防止内核态嵌套抢占导致栈溢出，Phoenix在发生抢占后会在当前的硬件上下文中设置标志位，从而不允许再次发生抢占，避免了嵌套抢占的情况。因此，Phoenix内核栈的大小最多只需要是原来栈大小的两倍。当前，内核栈的大小设置为64KB，已经足够大了。相比于传统的有栈协程需要为每个进程设置内核栈，即使在抢占式调度下需要增加原来内核栈的大小，无栈协程仍能保持空间利用的优势。


```rust
/// Kernel trap handler
#[no_mangle]
pub fn kernel_trap_handler() {
    let scause = scause::read();
    match scause.cause() {
        Trap::Interrupt::SupervisorTimer => {
            TIMER_MANAGER.check();
            unsafe { set_next_timer_irq() };
            #[cfg(feature = "preempt")]
            {
                if !executor::has_task() {
                    return;
                }
                // kernel preempt
                let mut old_hart = local_hart().enter_preempt_switch();
                executor::run_one();
                local_hart().leave_preempt_switch(&mut old_hart);
                // kernel preempt finished
            }
        },
        _ => ...
    }
}
```
