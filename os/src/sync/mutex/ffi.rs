use core::ops::{Deref, DerefMut};

use crate::hal::arch::interrupt::InterruptGuard;
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

pub struct SendWrapper<T>(pub T);

impl<T> SendWrapper<T> {
    pub fn new(data: T) -> Self {
        SendWrapper(data)
    }
}

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T: Deref> Deref for SendWrapper<T> {
    type Target = T::Target;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: DerefMut> DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}