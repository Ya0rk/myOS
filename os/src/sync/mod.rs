mod interrupt;
mod up;
pub mod mutex;
mod misc;
mod time;
pub mod timer;
pub mod once;

use core::{future::Future, task::{Context, Poll}};
use alloc::{boxed::Box, sync::Arc, task::Wake};

pub use interrupt::{enable_interrupt, disable_interrupt, interrupt_is_enabled, enable_timer_interrupt};
pub use mutex::{new_shared, new_sleep_shared};
pub use timer::{get_time, sleep_for, set_next_trigger};
pub use misc::{get_waker, yield_now, suspend_now};
pub use up::SyncUnsafeCell;
pub use time::{TimeVal, TimeSepc, Tms, TimeData, TimeStamp};
pub use mutex::{SpinLock, SpinNoIrqLock, SleepLock, MutexGuard, MutexOperations, NoIrqLock, NoopLock, Shared, SleepShared};


struct BlockWaker;

impl Wake for BlockWaker {
    fn wake(self: Arc<Self>) {}
}

/// 阻塞当前线程直到 future 执行完成
///
/// future 不会被调度，而是一直被轮询直到返回 Ready
pub fn block_on<T>(fut: impl Future<Output=T>) -> T {
    let mut fut = Box::pin(fut);

    let waker = Arc::new(BlockWaker).into();
    let mut ctx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut ctx) {
            Poll::Ready(res) => return res,
            Poll::Pending => continue,
        }
    }
}