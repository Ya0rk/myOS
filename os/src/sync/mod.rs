// mod interrupt;
mod misc;
pub mod mutex;
pub mod once;
pub mod time;
pub mod time_async;
pub mod timer;
mod up;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, sync::Arc, task::Wake, vec, vec::Vec};
use core::{
    future::Future,
    task::{Context, Poll},
    time::Duration,
};
use spin::Lazy;
use time::{CLOCK_MONOTONIC, CLOCK_REALTIME};

pub use crate::hal::arch::interrupt::{
    disable_supervisor_interrupt, enable_supervisor_interrupt, enable_supervisor_timer_interrupt,
    supervisor_interrupt_is_enabled,
};
pub use misc::{get_waker, suspend_now, yield_now};
pub use mutex::{new_shared, new_sleep_shared};
pub use mutex::{
    MutexGuard, MutexOperations, NoIrqLock, NoopLock, Shared, SleepLock, SleepShared, SpinLock,
    SpinNoIrqLock,
};
pub use time::{TimeData, TimeSpec, TimeStamp, TimeVal, Tms};
pub use time_async::*;
pub use timer::{set_next_trigger, sleep_for, time_duration};
pub use up::SyncUnsafeCell;

struct BlockWaker;

impl Wake for BlockWaker {
    fn wake(self: Arc<Self>) {}
}

/// 阻塞当前线程直到 future 执行完成
///
/// future 不会被调度，而是一直被轮询直到返回 Ready
pub fn block_on<T>(fut: impl Future<Output = T>) -> T {
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

/// Clock manager that used for looking for a given process
pub static CLOCK_MANAGER: Lazy<SpinNoIrqLock<Vec<Duration>>> =
    Lazy::new(|| SpinNoIrqLock::new(Vec::new()));

pub fn time_init() {
    CLOCK_MANAGER.lock().insert(CLOCK_REALTIME, Duration::ZERO);

    CLOCK_MANAGER.lock().insert(CLOCK_MONOTONIC, Duration::ZERO);
}
