use crate::sync::interrupt::InterruptGuard;
use super::MutexOperations;


pub struct NoopLock;

impl MutexOperations for NoopLock {
    type GuardData = ();
    #[inline(always)]
    fn before_lock() -> Self::GuardData {}
    #[inline(always)]
    fn after_unlock(_: &mut Self::GuardData) {}
}

pub struct NoIrqLock;

impl MutexOperations for NoIrqLock {
    type GuardData = InterruptGuard;
    #[inline(always)]
    fn before_lock() -> Self::GuardData {
        InterruptGuard::new()
    }
    #[inline(always)]
    fn after_unlock(_: &mut Self::GuardData) {}
}