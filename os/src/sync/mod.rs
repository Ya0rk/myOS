// mod interrupt;
mod up;
mod misc;
pub mod mutex;
pub mod time;
pub mod timer;
pub mod once;
pub mod time_async;

use core::{future::Future, task::{Context, Poll}, time::Duration};
use alloc::{boxed::Box, collections::btree_map::BTreeMap, sync::Arc, task::Wake, vec::Vec, vec};
use spin::Lazy;
use time::{CLOCK_MONOTONIC, CLOCK_REALTIME};

pub use crate::hal::arch::interrupt::{enable_interrupt, disable_interrupt, interrupt_is_enabled, enable_timer_interrupt};
pub use mutex::{new_shared, new_sleep_shared};
pub use timer::{sleep_for, set_next_trigger, time_duration};
pub use misc::{get_waker, yield_now, suspend_now};
pub use up::SyncUnsafeCell;
pub use time_async::*;
pub use time::{TimeVal, TimeSpec, Tms, TimeData, TimeStamp};
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

/// Clock manager that used for looking for a given process
pub static CLOCK_MANAGER: Lazy<SpinNoIrqLock<Vec<Duration>>> = Lazy::new(|| SpinNoIrqLock::new(Vec::new()));

pub fn time_init() {
    CLOCK_MANAGER
        .lock()
        .insert(CLOCK_REALTIME, Duration::ZERO);
    
    CLOCK_MANAGER
        .lock()
        .insert(CLOCK_MONOTONIC, Duration::ZERO);
}