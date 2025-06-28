= 时钟模块
<时钟模块>

== 时钟中断
<时钟中断>

RISC-V架构中，内置时钟和计数器用于实现操作系统的计时机制，其中64位的`mtime`计数器记录处理器自上电以来的时钟周期，而`mtimecmp`用于触发时钟中断。由于内核处于S特权级，无法直接访问这些M特权级的CSR，因此通过SEE（OpenSBI）提供的SBI接口间接实现计时器控制。

Phoenix中利用`riscv::register::time::read()`函数读取 RISC-V 架构下的
`mtime`
计数器的值，得到系统自启动以来经过的时钟周期数作为系统时间，转换为Rust核心库`core`中的`Duration`结构体，它能够清晰地表示时间间隔，避免了直接操作裸露的计数值所带来的错误和混淆，确保时间的计算和表示是统一的，并且可以利用
`Duration` 提供的丰富的时间操作方法（如加减法、比较等）

== 定时器
<定时器>

在操作系统中，定时器通常用来管理一段时间后需要触发的事件。这些定时器需要记录触发时间和要执行的回调函数。

Phoenix的定时器结构体`Timer`参考了Linux系统的回调函数设计，但是结合了rust的特性。Phoenix定义了`TimerEvent`
trait，定义了一个通用的接口，用于描述定时器触发时需要执行的操作。与往届作品Titanix仅在`Timer`中定义了`Waker`用于唤醒相比，这种设计提高了定时器的灵活性。

```rust
/// A trait that defines the event to be triggered when a timer expires.
/// The TimerEvent trait requires a callback method to be implemented,
/// which will be called when the timer expires.
pub trait TimerEvent: Send + Sync {
    /// The callback method to be called when the timer expires.
    /// This method consumes the event data and optionally returns a new
    /// timer.
    ///
    /// # Returns
    /// An optional Timer object that can be used to schedule another timer.
    fn callback(self: Box<Self>) -> Option<Timer>;
}
```

`callback`方法的参数`self: Box<Self>`通过将 `self` 移动到 `Box`
内，保证了 trait 对象的动态分发能力（即运行时多态），并且确保调用
`callback`
时定时器的数据所有权被安全转移。返回值为`Option<Timer>`，表示在当前定时器触发后，可以选择性地创建一个新的定时器，这种设计使得定时器能够链式触发，以便支持需要重复触发定时器的`sys_setitimer`系统调用。通过
`Send` 和 `Sync` trait
bounds，确保定时器事件在多线程环境中是安全的。可以在线程间传递和共享
`Timer` 实例，而无需担心数据竞争问题。

`Timer`
结构体用来表示一个具体的定时器实例，包含到期时间和需要执行的回调函数。具体设计如下：

```rust
/// Represents a timer with an expiration time and associated event data.
/// The Timer structure contains the expiration time and the data required
/// to handle the event when the timer expires.
pub struct Timer {
    /// The expiration time of the timer.
    /// This indicates when the timer is set to trigger.
    pub expire: Duration,

    /// A boxed dynamic trait object that implements the TimerEvent trait.
    /// This allows different types of events to be associated with the
    /// timer.
    pub data: Box<dyn TimerEvent>,
}
```

== 定时器队列
<定时器队列>

Phoenix使用`TimerManager`
结构体实现了一个高效、安全且易于管理的定时器管理机制。使用`BinaryHeap`二叉堆数据结构按到期时间排序管理所有的定时器，其插入和删除操作复杂度为
O(log n)，能在 O(1)
时间内获取最早到期的定时器，确保定时器触发的实时性。Phoenix支持用户态时间中断和内核态时间中断，两种中断触发时都会检查是否有定时器到期。

```rust
/// `TimerManager` is responsible for managing all the timers in the system.
pub struct TimerManager {
    /// A priority queue to store the timers.
    timers: SpinNoIrqLock<BinaryHeap<Reverse<Timer>>>,
}
```

目前Phoenix已实现较为高效的定时任务管理，但是仍有可以改进的地方，例如当前使用的`BinaryHeap`数据结构虽然在插入和删除操作上的复杂度较低，但由于其需要频繁分配和释放内存，可能会导致性能上的开销。而侵入式链表（如Linux中的`list_head`）可以减少内存分配和释放的频率，Linux采用侵入式链表与红黑树实现了更高效的定时任务管理。
