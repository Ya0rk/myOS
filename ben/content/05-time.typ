#import "../components/prelude.typ": *
= 时钟模块
<时钟模块>

== 定时器队列
<定时器队列>

在操作系统的定时器模块中，我们创造性地实现了一套混合定时器管理系统，将时间轮算法与最小堆优化相结合，相较于Phoenix和Pantheon使用的最小堆数据结构，我们的混合定时器在时间的处理上更加精细，同时在特定场景下更加的高效。

我们将定时器分为了短期和长期两种，以600ms作为阈值，将小于阈值的划分为短期定时器，大于阈值的划分为长期定时器。两种定时器分别存储在时间轮和最小堆中。

#code-figure(
```rs
// 定时器队列
pub struct TimerQueue {
    /// 短期定时器（<600ms）
    wheel: SpinNoIrqLock<TimingWheel>,
    /// 长期定时器（最小堆）
    long_term: SpinNoIrqLock<BinaryHeap<TimerEntry>>,
    /// 定时器句柄计数器
    handle_counter: SpinNoIrqLock<u64>,
}
```,
    caption: [TimerQueue 结构],
  label-name: "TimeQueue 结构",
)


=== 时间轮设计
<时间轮设计>

时间轮是一种高效的定时器管理数据结构，特别适合处理大量短周期定时器。鉴于我们内核的时钟中断间隔为10ms，所以将时间轮划分为60个槽位，同时时间轮的滴答间隔为10ms， 10ms自动推进一槽，与硬件时钟中断完美同步。如下图所示，当时间指针指向的槽位为1时，代表这次推进将处理1槽位中所有的定时器。槽内采用平铺向量存储，插入/删除操作达到O(1)常数时间，相较于最小堆插入O(log n)/删除O(log n)更加高效。

#code-figure(
```rs
struct TimingWheel {
    // 时间轮的槽数组，每个槽存储一组定时器条目
    slots: [Vec<TimerEntry>; TIME_WHEEL_SLOTS],
    // 当前指向的槽索引，随着时间推移循环递增
    current_slot: usize,
    // 时间轮当前表示的时间点
    current_time: Duration,
}
```,
    caption: [TimingWheel 结构体],
    label-name: "timing-wheel-struct",
)

#figure(
  image("assets/timingwhell.png"),
  caption: [时间轮结构],
  supplement: [图],
)<时间轮结构>

时间轮推进算法如下所示，通过该函数我们获取到所有超时的定时器waker，返回给上级调用者用于批量唤醒。

#algorithm-figure(
  pseudocode(
    no-number,
    [*function* advance_to(target_time: Duration) $\to$ Vec<Waker>],
    [wake_list ← 空列表],
    [计算需要推进的槽数 slots_to_advance = calc_slot(target_time)],
    [*for* 每个需要处理的槽 *do*],
    ind,
    [*for* 当前槽中的每个定时器条目 *do*],
    ind,
    [*if* 条目已过期(expire ≤ target_time) *then*],
    ind,
    [将条目的waker加入唤醒列表],
    [从槽中移除该条目],
    ded,
    [*else*],
    ind,
    [保留该条目],
    ded,
    ded,
    [移动指针到下一个槽],
    [更新时间轮当前时间],
    [*if* 当前时间超过目标时间 *then* 提前退出],
    ded,
    [*return* wake_list],
  ),
  caption: [时间轮推进算法],
  label-name: "time-wheel-advance-simplified",
  supplement: [算法]
)

== 定时器
<定时器>

我们使用TimeEntry表示定时器，每个定时器都携带专属的TimerHandle——一个单调递增的唯一标识符。当异步Future结束其生命周期时，我们需要drop其中剩下时的定时器，这时就可以通过对比TimerHandle找到对应的定时器。系统先在时间轮中闪电扫描，然后扫描二叉堆。为优化堆内搜索，我们设计了临时缓存策略：将非目标项暂存后重新入堆，避免了重建整个堆的昂贵开销。这种双路径检索确保删除操作始终保持高效。

#code-figure(
```rust
// 定时器条目
struct TimerEntry {
    /// 到期时间
    expire: Duration,
    /// 唤醒器
    waker: Option<Waker>,
    /// 用于取消的句柄
    handle: TimerHandle,
}
```,
    caption: [TimerEntry 结构体],
    label-name: "timer-entry-struct",
)

为了充分利用异步的优势，我们将需要监测的任务被封装在一个超时Future中（如下所示），deadline表示任务的超时期限，timer_handle用于Drop机制中确保定时器资源的回收，即使任务提前完成也不会留下幽灵定时器。我们使用poll轮循的方式对任务进行推进检测，轮询时执行三重检测：首先尝试推进内部任务，其次检查期限是否届满，最后才注册唤醒器，并且唤醒器只会注册一次。

#code-figure(
```rust
pub struct TimeoutFuture<F: Future> {
    inner: F,
    deadline: Duration,
    timer_handle: Option<TimerHandle>,
}

impl<F: Future> Drop for TimeoutFuture<F> {
    fn drop(&mut self) {
        // 确保定时器被删除
        if let Some(handle) = self.timer_handle.take() {
            TIMER_QUEUE.cancel(handle);
        }
    }
}
```,
    caption: [TimeoutFuture 结构体与 Drop 实现],
    label-name: "timeout-future-drop",
)

目前Del0n1x已实现较为高效的定时任务管理，但是该优化并没有在初赛的测试用例中体现出来，在初赛测试用例中大部分定时器时间为1s，其实都会被分配到最小堆结构中。这也反向说明了对于代码的优化也要在特定的场景中才能得到体现，比如时间轮适合处理大量短周期定时任务，最小堆适合处理少量长周期定时任务。