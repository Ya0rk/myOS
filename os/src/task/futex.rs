use super::{current_task, Pid, TaskControlBlock};
use crate::mm::{Direct, PhysAddr, VirtAddr};
use crate::sync::{SpinNoIrqLock, SyncUnsafeCell};
use crate::utils::{Errno, SysResult};
use alloc::collections::btree_map::BTreeMap;
use alloc::task::Wake;
use alloc::{sync::Arc, vec::Vec};
use core::cell::UnsafeCell;
use core::cmp::min;
use core::future::Future;
use core::intrinsics::{atomic_load_acquire, atomic_load_relaxed};
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Poll, Waker};
use hashbrown::HashMap;
use log::info;
use spin::Lazy;

pub const FUTEX_BITSET_MATCH_ANY: u32 = 0xffffffff;

/// 用于计算hashkey，然后在全局hash桶中获取到对应的futex hash链表
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum FutexHashKey {
    Shared { addr: PhysAddr },
    Privite { addr: VirtAddr },
}

impl FutexHashKey {
    pub fn get_futex_key(uaddr: usize, flags: FutexOp) -> FutexHashKey {
        let va = VirtAddr(uaddr);
        if flags.contains(FutexOp::FUTEX_PRIVATE) {
            // let task = current_task().unwrap();
            // let mm = Arc::as_ptr(&task.memory_space) as usize;
            // return FutexHashKey::Privite { vaddr: va, mm };
            return FutexHashKey::Privite { addr: va };
        }

        // TODO: 正确吗？ by lsz
        let pa = va.direct_pa();
        return FutexHashKey::Shared { addr: pa };
    }
}

/// 全局futex hash桶，管理着所有的futex
// pub static FUTEXBUCKET: Lazy<SpinNoIrqLock<FutexBucket>> = Lazy::new(|| SpinNoIrqLock::new(FutexBucket::new()));

/// 每个hash key对应一个vec，其中是(线程id，进程waker, bitset位掩码）三元组，waker用来唤醒进程
pub struct FutexBucket(pub HashMap<FutexHashKey, UnsafeCell<Vec<(usize, Waker, u32)>>>);

impl FutexBucket {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// 获取不可变的futex hash桶
    pub fn get_values(&self, key: FutexHashKey) -> Option<&Vec<(usize, Waker, u32)>> {
        self.0.get(&key).map(|v| unsafe { &*v.get() })
    }

    /// 获取可变的futex hash桶
    pub fn get_mut_values(&mut self, key: FutexHashKey) -> Option<&mut Vec<(usize, Waker, u32)>> {
        self.0.get_mut(&key).map(|v| unsafe { &mut *v.get() })
    }

    pub fn check_is_inqueue(&self, key: FutexHashKey, pid: usize) -> bool {
        match self.get_values(key) {
            Some(queue) => queue.iter().any(|(p, _, _)| *p == pid),
            None => false,
        }
    }

    pub fn add(&mut self, key: FutexHashKey, pid: usize, waker: Waker, bitset: u32) {
        match self.get_mut_values(key) {
            Some(queue) => {
                queue.push((pid, waker, bitset));
            }
            None => {
                let mut new_queue = Vec::new();
                new_queue.push((pid, waker, bitset));
                self.0.insert(key, UnsafeCell::new(new_queue));
            }
        }
    }

    /// 用于删除特定的futex，主要场景分为两种：超时或者被信号打断
    pub fn remove(&mut self, key: FutexHashKey, pid: usize) {
        info!("[futex queue] remove pid = {}", pid);
        let queue = match self.get_mut_values(key) {
            Some(queue) => queue,
            None => return,
        };
        queue.retain(|(p, _, _)| *p != pid); // 删除队列中pid的任务
        if queue.is_empty() {
            // 队列为空，就删除整个队列，避免内存泄露
            self.0.remove(&key);
        }
    }

    // 唤醒在队列中的任务, 同时将这些任务从队列中清除
    pub fn to_wake(&mut self, key: FutexHashKey, bitset: u32, num: u32) -> usize {
        if num == 0 {
            return 0;
        }
        let mut res = 0;
        if let Some(queue) = self.get_mut_values(key) {
            while let Some((pid, waker, tb)) = queue.pop() {
                if bitset & tb == 0 {
                    queue.push((pid, waker, tb)); // 如果不满足条件，就放回去
                    println!("[to_wake] no reqqq");
                    continue;
                }
                waker.wake();
                res += 1;
                if res >= num as usize {
                    break;
                }
            }
            if queue.is_empty() {
                self.0.remove(&key);
            }
        }
        return res;
    }

    /// 将key中剩下的futex移动到newkey队列中
    pub fn requeue(
        &mut self,
        key: FutexHashKey,
        new_key: FutexHashKey,
        max_num: usize,
    ) -> SysResult<()> {
        let mut should_remove = false;
        let mut migrated = None;

        // 处理旧队列
        if let Some(uc) = self.0.get_mut(&key) {
            let old_queue = unsafe { &mut *uc.get() };
            let have = old_queue.len();
            let do_len = min(have, max_num);

            if do_len == 0 {
                // 无元素可迁移，检查是否需删除空队列
                should_remove = old_queue.is_empty();
            } else {
                // 迁移元素并检查是否需删除旧键
                migrated = Some(old_queue.split_off(have - do_len));
                should_remove = old_queue.is_empty();
            }
        } else {
            // 旧队列不存在，直接返回成功
            return Ok(());
        }

        // 清理空队列
        if should_remove {
            self.0.remove(&key);
        }

        // 迁移到新队列
        if let Some(mut migrated) = migrated {
            let new_entry = self
                .0
                .entry(new_key)
                .or_insert_with(|| UnsafeCell::new(Vec::new()));
            let new_queue = unsafe { &mut *new_entry.get() };
            new_queue.append(&mut migrated);
        }

        Ok(())
    }
}

pub struct FutexFuture {
    pub uaddr: usize,
    pub key: FutexHashKey,
    pub bitset: u32, // 位掩码，用于唤醒判断
    pub val: u32,    // 期望的值，在入队列前需要判断是否相等
    pub is_register: bool,
    // pub task: Arc<TaskControlBlock>,
}

impl FutexFuture {
    pub fn new(uaddr: usize, key: FutexHashKey, bitset: u32, val: u32) -> Self {
        FutexFuture {
            uaddr,
            key,
            bitset,
            val,
            is_register: false,
            // task: current_task().unwrap(),
        }
    }
}

impl Future for FutexFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.get_mut();
        // let task = this.task.as_ref();
        let task = current_task().unwrap();
        let pid = task.get_pid();
        let uaddr = this.uaddr;
        info!(
            "[futex_future] poll pid = {}, uaddr = {:#x}, is_register = {}, self.val = {}",
            pid, uaddr, this.is_register, this.val
        );
        // println!("[futex_future] poll pid = {}, uaddr = {:#x}, is_register = {}, self.val = {}", pid, uaddr, this.is_register, this.val);
        if !this.is_register {
            // 说明还没有加入全局hash 桶
            // 在入队前要判断是否是期望值
            if unsafe { atomic_load_acquire(uaddr as *const u32) } != this.val {
                // 如果不相等，说明有其他任务释放了锁，此时就不需要将其加入队列，否则可能造成无法唤醒
                info!("[futex_future] now != val");
                return Poll::Ready(());
            }
            // if this.uaddr == 0x4008004d0 && this.val == 6 {
            //     task.set_zombie();
            //     return Poll::Ready(());
            // }
            // if this.uaddr == 0x1007ffbe0 && (this.val == 2147483655 || this.val == 2147483654) {
            //     task.set_zombie();
            //     return Poll::Ready(());
            // }
            // if this.uaddr == 0x1007ffc10 && this.val == 0 {
            //     task.set_zombie();
            //     return Poll::Ready(());
            // }

            // 加入hash 桶
            task.futex_list
                .lock()
                .add(this.key, pid, cx.waker().clone(), this.bitset);
            this.is_register = true;
            return Poll::Pending;
        }

        task.futex_list.lock().remove(this.key, pid);
        return Poll::Ready(());
    }
}

pub const ROBUST_LIST_HEAD_SIZE: usize = 24;

/// 用来记录用户态链表，用户通过set_robust_list向内核注册一个链表地址
/// 内核用下面结构体来记录
pub struct RobustList {
    pub head: usize,
}

impl RobustList {
    pub fn new() -> Self {
        Self { head: 0 }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct FutexOp: i32 {
        // 基础操作 (低4位定义操作类型)
        /// 等待操作：检查*uaddr是否等于期望值，若相等则挂起线程
        /// 场景：实现锁等待/条件变量
        const FUTEX_WAIT	    =	0;  // 0x0

        /// 唤醒操作：唤醒最多指定数量的等待线程
        /// 场景：释放锁时通知等待者
        const FUTEX_WAKE	    =	1;  // 0x1

        /// （已废弃）通过文件描述符关联futex，Linux 2.6.25+ 已移除
        #[deprecated = "FUTEX_FD 已被废弃，请使用其他替代方案"]
        const FUTEX_FD        =   2;  // 0x2

        /// 重新排队：将等待线程从一个队列迁移到另一个队列
        /// 场景：减少锁竞争（如线程池任务转移）
        const FUTEX_REQUEUE		= 3;  // 0x3

        /// 带条件检查的重新排队：需验证*uaddr值后再迁移
        /// 场景：安全的队列迁移（避免竞态）
        const FUTEX_CMP_REQUEUE	= 4;  // 0x4

        /// 原子操作后唤醒：执行自定义原子操作后唤醒线程
        /// 场景：实现读写锁等复杂同步
        const FUTEX_WAKE_OP		= 5;  // 0x5

        /// 获取优先级继承(PI)锁：解决优先级反转问题
        /// 场景：实时系统/高优先级线程需要优先执行
        const FUTEX_LOCK_PI		= 6;  // 0x6

        /// 释放优先级继承(PI)锁
        const FUTEX_UNLOCK_PI		= 7;  // 0x7

        /// 尝试获取PI锁（非阻塞）
        const FUTEX_TRYLOCK_PI	= 8;  // 0x8

        /// 位掩码等待：通过bitset指定唤醒条件
        const FUTEX_WAIT_BITSET	= 9;  // 0x9

        const FUTEX_WAKE_BITSET   = 10;

        const FUTEX_PRIVATE       = 128;  // 0x80

        /// 使用实时时钟：超时基于CLOCK_REALTIME
        /// 用法：FUTEX_WAIT | FUTEX_CLOCK_REALTIME
        const FUTEX_CLOCK_REALTIME= 256;  // 0x100

        const FUTEX_MUSK = 0x180; // 用于清除上面两个标志位
    }
}
