use core::{cell::UnsafeCell, marker::PhantomData, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use super::{ffi::SendWrapper, MutexOperations};

pub struct MutexGuard<'a, T: ?Sized, S: MutexOperations> {
    mutex: &'a SpinMutex<T, S>,
    support_guard: S::GuardData,
}

// Forbid Mutex step over `await` and lead to dead lock
impl<'a, T: ?Sized, S: MutexOperations> !Sync for MutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexOperations> !Send for MutexGuard<'a, T, S> {}

pub struct SpinMutex<T: ?Sized, L:MutexOperations> {
    _marker: core::marker::PhantomData<L>,
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, S: MutexOperations> Sync for SpinMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexOperations> Send for SpinMutex<T, S> {}

impl<T, S: MutexOperations> SpinMutex<T, S> {
    /// Construct a SpinMutex
    pub const fn new(user_data: T) -> Self {
        SpinMutex {
            lock: AtomicBool::new(false),
            _marker: PhantomData,
            data: UnsafeCell::new(user_data),
        }
    }
    /// Wait until the lock looks unlocked before retrying
    #[inline(always)]
    fn wait_unlock(&self) {
        let mut try_count = 0usize;
        while self.lock.load(Ordering::Relaxed) {
            core::hint::spin_loop();
            try_count += 1;
            if try_count == 0x1000000000 {
                panic!("Mutex: deadlock detected! try_count > {:#x}\n", try_count);
            }
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn lock(&self) -> MutexGuard<T, S> {
        let support_guard = S::before_lock();
        loop {
            self.wait_unlock();
            if self
                .lock
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        MutexGuard {
            mutex: self,
            support_guard,
        }
    }

    pub unsafe fn sent_lock(&self) -> impl DerefMut<Target = T> + '_ {
        SendWrapper::new(self.lock())
    }

}

impl<'a, T: ?Sized, S: MutexOperations> Deref for MutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexOperations> DerefMut for MutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexOperations> Drop for MutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created
    /// from.
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}
