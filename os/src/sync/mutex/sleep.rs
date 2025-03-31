use alloc::{collections::VecDeque, sync::Arc};
use core::{
    cell::{SyncUnsafeCell, UnsafeCell},
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
};
use crate::sync::get_waker;
use super::{spin::SpinMutex, MutexOperations};

/// SleepMutex can step over `await`
pub struct SleepMutex<T: ?Sized, S: MutexOperations> {
    lock: SpinMutex<MutexInner, S>, // push at prev, release at next
    data: UnsafeCell<T>,            // actual data
}

struct MutexInner {
    is_locked: bool,
    // 等待队列
    wait_queue: UnsafeCell<Option<VecDeque<Arc<GrantInfo>>>>,
}

unsafe impl<T: ?Sized + Send, S: MutexOperations> Send for SleepMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexOperations> Sync for SleepMutex<T, S> {}

impl<T, S: MutexOperations> SleepMutex<T, S> {
    /// 新建一个睡眠锁
    pub const fn new(user_data: T) -> Self {
        SleepMutex {
            lock: SpinMutex::new(MutexInner {
                is_locked: false,
                wait_queue: UnsafeCell::new(None),
            }),
            // _marker: PhantomData,
            data: UnsafeCell::new(user_data),
        }
    }
}

impl<T: ?Sized + Send, S: MutexOperations> SleepMutex<T, S> {
    /// Lock
    #[inline]
    pub async fn lock(&self) -> impl DerefMut<Target = T> + Send + Sync + '_ {
        let future = &mut SleepMutexFuture::new(self);
        unsafe { Pin::new_unchecked(future).init().await.await }
    }
}

// struct GrantInfo {
//     inner: SyncUnsafeCell<(AtomicBool, Option<Waker>)>,
// }

struct GrantInfo {
    is_granted: AtomicBool,
    waker: SyncUnsafeCell<Option<Waker>>,
}

struct SleepMutexFuture<'a, T: ?Sized, S: MutexOperations> {
    mutex: &'a SleepMutex<T, S>,
    grant: Arc<GrantInfo>,
}

impl<'a, T: ?Sized, S: MutexOperations> SleepMutexFuture<'a, T, S> {
    #[inline(always)]
    fn new(mutex: &'a SleepMutex<T, S>) -> Self {
        SleepMutexFuture {
            mutex,
            grant: Arc::new(GrantInfo {
                is_granted: AtomicBool::new(false),
                waker: SyncUnsafeCell::new(None),
            }),
        }
    }

    async fn init(self: Pin<&mut Self>) -> Pin<&mut SleepMutexFuture<'a, T, S>> {
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { &mut *this.mutex.lock.sent_lock() };
        if !inner.is_locked {
            // The sleep lock is not yet locked, just granted.
            inner.is_locked = true;
            // unsafe { &mut *this.grant.inner.get() }
            //     .0
            //     .store(true, Ordering::Release);
            this.grant.is_granted.store(true, Ordering::Release);
        } else {
            log::trace!("[SleepMutexFuture::init] wait for lock...");
            let waker = Some(get_waker().await);
            unsafe { *this.grant.waker.get() = waker }; // 锁已经被占用则挂起任务
            let queue = unsafe { &mut (*inner.wait_queue.get()) };
            if queue.is_none() {
                *queue = Some(VecDeque::new());
            }
            queue.as_mut().unwrap().push_back(this.grant.clone());
        }
        unsafe { Pin::new_unchecked(this) }
    }
}

impl<'a, T: ?Sized, S: MutexOperations> Future for SleepMutexFuture<'a, T, S> {
    type Output = SleepMutexGuard<'a, T, S>;
    #[inline(always)]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.grant.is_granted.load(Ordering::Acquire) {
            false => Poll::Pending,
            true => {
                log::trace!("[SleepMutexFuture::poll] granted");
                Poll::Ready(SleepMutexGuard { mutex: self.mutex })
            }
        }
    }
}

struct SleepMutexGuard<'a, T: ?Sized, S: MutexOperations> {
    mutex: &'a SleepMutex<T, S>,
}

unsafe impl<'a, T: ?Sized + Send, S: MutexOperations> Send for SleepMutexGuard<'a, T, S> {}
unsafe impl<'a, T: ?Sized + Send, S: MutexOperations> Sync for SleepMutexGuard<'a, T, S> {}

impl<'a, T: ?Sized, S: MutexOperations> Deref for SleepMutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexOperations> DerefMut for SleepMutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexOperations> Drop for SleepMutexGuard<'a, T, S> {
    #[inline]
    fn drop(&mut self) {
        log::trace!("[SleepMutexGuard::drop] drop...");
        let mut inner = self.mutex.lock.lock();
        debug_assert!(inner.is_locked);

        if let Some(queue) = unsafe { &mut *inner.wait_queue.get() } {
            if let Some(waiter) = queue.pop_front() {
                // Wake the next task
                let waker = unsafe { &mut *waiter.waker.get() }.take().unwrap();
                waiter.is_granted.store(true, Ordering::Release);
                waker.wake();
                return;
            }
        }

        // No waiters, release the lock
        inner.is_locked = false;

        // let queue = unsafe { &mut (*inner.wait_queue.get()) };
        // if queue.is_none() {
        //     inner.is_locked = false;
        //     log::trace!("[SleepMutexGuard::drop] queue is none");
        //     return;
        // }
        // let waiter = match queue.as_mut().unwrap().pop_front() {
        //     None => {
        //         // The wait queue is empty
        //         inner.is_locked = false;
        //         log::trace!("[SleepMutexGuard::drop] queue is empty");
        //         return;
        //     }
        //     Some(waiter) => waiter,
        // };
        // drop(inner);
        // // Waker should be fetched before we make the grant_inner.0 true
        // // since it will be invalid after that.
        // let grant_inner = unsafe { &mut *waiter.inner.get() };
        // let waker = grant_inner.1.take().unwrap();
        // grant_inner.0.store(true, Ordering::Release);
        // waker.wake();
        // log::trace!("[SleepMutexGuard::drop] grant someone...");
    }
}
