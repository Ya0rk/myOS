mod ffi;
mod spin;
mod sleep;

use alloc::sync::Arc;
pub use spin::MutexGuard;
pub use ffi::{NoIrqLock, NoopLock};

pub type SpinLock<T> = spin::SpinMutex<T, ffi::NoopLock>;
pub type SpinNoIrqLock<T> = spin::SpinMutex<T, ffi::NoIrqLock>;
pub type SleepLock<T> = sleep::SleepMutex<T, ffi::NoIrqLock>;
pub type Shared<T> = Arc<SpinNoIrqLock<T>>;
pub type SleepShared<T> = Arc<SleepLock<T>>;


pub trait MutexOperations {
    /// Guard data
    type GuardData;
    /// Called before lock() & try_lock()
    fn before_lock() -> Self::GuardData;
    /// Called when MutexGuard dropping
    fn after_unlock(_: &mut Self::GuardData);
}

pub fn new_shared<T>(data: T) -> Shared<T> {
    Arc::new(SpinNoIrqLock::new(data))
}

pub fn new_sleep_shared<T>(data: T) -> SleepShared<T> {
    Arc::new(SleepLock::new(data))
}