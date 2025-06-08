use crate::{
    mm::VirtAddr,
    sync::{get_waker, suspend_now, yield_now, TimeSpec, TimeoutFuture},
    task::{
        current_task, get_task_by_pid, FutexFuture, FutexHashKey, FutexOp, Pid,
        FUTEX_BITSET_MATCH_ANY, ROBUST_LIST_HEAD_SIZE,
    },
    utils::{Errno, SysResult},
};
use alloc::task;
use core::{
    intrinsics::{atomic_load_acquire, atomic_load_relaxed},
    time::Duration,
};
use log::info;
use num_enum::TryFromPrimitive;

/// fast user-space locking
/// uaddr就是用户态下共享内存的地址，里面存放的是一个对齐的整型计数器。
/// val:可以表示期待的值；在wake中表示唤醒的数量
/// op存放着操作类型。定义的有5中，这里我简单的介绍一下两种
/// FUTEX_WAIT: 原子性的检查uaddr中计数器的值是否为val,如果是则让进程休眠，直到FUTEX_WAKE或者超时(time-out)。也就是把进程挂到uaddr相对应的等待队列上去。
/// FUTEX_WAKE: 最多唤醒val个等待在uaddr上进程。
/// uaddr2: 第二个用户空间地址（用于某些复杂操作，如 FUTEX_REQUEUE）
/// val3: 在wait中表示位掩码；在wake中需要用来和uaddr做判断
pub async fn sys_futex(
    uaddr: usize,
    futex_op: i32,
    val: u32,
    timeout: usize, // val2
    uaddr2: usize,
    val3: u32,
) -> SysResult<usize> {
    let mut op = FutexOp::from_bits(futex_op).ok_or(Errno::EINVAL)?;
    info!("[sys_futex] start, futex_op = {:?}", op);
    if uaddr == 0 {
        return Err(Errno::EACCES);
    }
    let key = FutexHashKey::get_futex_key(uaddr, op);
    // 按照linux做一些判断
    if op.contains(FutexOp::FUTEX_CLOCK_REALTIME) {
        let cmd = op & !FutexOp::FUTEX_MUSK;
        if cmd != FutexOp::FUTEX_WAIT_BITSET {
            return Err(Errno::ENOSYS);
        }
    }
    let use_op = op & !FutexOp::FUTEX_MUSK;
    info!("[sys_futex] use_op = {:?}", use_op);

    match use_op {
        FutexOp::FUTEX_WAIT => {
            // // 如果futex word中仍然保存着参数val给定的值，那么当前线程则进入睡眠，等待FUTEX_WAKE的操作唤醒它。
            let now_val = unsafe { atomic_load_acquire(uaddr as *const u32) };
            info!(
                "[sys_futex] wait, uaddr = {:#x}, taskid = {}, now_val = {}, val = {}",
                uaddr,
                current_task().unwrap().get_pid(),
                now_val,
                val
            );
            // // 如果uaddr指向地址的值 != val，那么就返回错误
            if now_val != val {
                return Err(Errno::EAGAIN);
            }

            info!(
                "[sys_futex] wait, now_val = {}, val = {}, timeout = {}",
                now_val, val, timeout
            );
            let res = do_futex_wait(uaddr, val, timeout, FUTEX_BITSET_MATCH_ANY, key).await;
            return res;
        }
        FutexOp::FUTEX_WAIT_BITSET => {
            let now_val = unsafe { atomic_load_acquire(uaddr as *const u32) };
            if now_val != val {
                return Err(Errno::EAGAIN);
            }

            let res = do_futex_wait(uaddr, val, timeout, val3, key).await;
            return res;
        }
        FutexOp::FUTEX_WAKE => {
            // // 最多唤醒val个等待在futex word上的线程。Val或者等于1（唤醒1个等待线程）或者等于INT_MAX（唤醒全部等待线程）
            info!("[sys_futex] wake, val = {}", val);
            let res = do_futex_wake(uaddr, val, FUTEX_BITSET_MATCH_ANY, key);
            yield_now().await;
            return res;
        }
        FutexOp::FUTEX_WAKE_BITSET => {
            let res = do_futex_wake(uaddr, val, val3, key);
            yield_now().await;
            return res;
        }
        FutexOp::FUTEX_REQUEUE => {
            // 这个操作包括唤醒和移动队列两个动作。唤醒val个等待在uaddr上的waiter，如果还有其他的waiter，
            // 那么将这些等待在uaddr的waiter转移到uaddr2的等待队列上去（最多转移val2(timeout)个waiter）
            let wake_n = do_futex_wake(uaddr, val, FUTEX_BITSET_MATCH_ANY, key)?;
            if uaddr == uaddr2 {
                return Ok(wake_n);
            }
            let new_key = FutexHashKey::get_futex_key(uaddr2 as usize, op);
            // 将剩下的移动到newkey队列中
            let task = current_task().unwrap();
            task.futex_list.lock().requeue(key, new_key, timeout)?;
            return Ok(wake_n);
        }
        FutexOp::FUTEX_CMP_REQUEUE => {
            // 和上面一样，只是多使用了一个val3判断
            let now = unsafe { atomic_load_relaxed(uaddr as *const u32) };
            if now != val3 {
                return Err(Errno::EAGAIN);
            }
            let wake_n = do_futex_wake(uaddr, val, FUTEX_BITSET_MATCH_ANY, key)?;
            if uaddr == uaddr2 {
                return Ok(wake_n);
            }
            let new_key = FutexHashKey::get_futex_key(uaddr2 as usize, op);
            let task = current_task().unwrap();
            task.futex_list.lock().requeue(key, new_key, timeout)?;
            return Ok(wake_n);
        }

        _ => {
            unimplemented!()
        }
    }

    Ok(0)
}

/// 对于带超时的场景，futex wait会首先通过futex_setup_timer设置定时器。
/// 接下来调用futex_wait_setup函数，后者主要做了两件事，一是根据入参获取全局哈希表的key从而找到task所属的bucket并获取自旋锁；
/// 二是在入队之前最后判断uaddr是否为预期值。如果uaddr被更新即非预期值，则会重新返回用户态去抢锁。
/// 否则执行下一步，即调用futex_wait_queue_me。
/// 后者主要做了几件事：1、将当前的task插入等待队列；2、启动定时任务；3、触发重新调度。
/// 接下来当task能够继续执行时会判断自己是如何被唤醒的，并释放hrtimer退出。
async fn do_futex_wait(
    uaddr: usize,
    val: u32,
    timeout: usize,
    bitset: u32,
    key: FutexHashKey,
) -> SysResult<usize> {
    if bitset == 0 {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let wake_up_sig = *task.get_blocked();
    task.set_wake_up_signal(wake_up_sig);

    let futex_future = FutexFuture::new(uaddr, key, bitset, val);
    info!("[do_futex_wait] uaddr = {:#x}", uaddr);

    match timeout {
        // 此时不需要等待，立即执行
        0 => futex_future.await,
        _ => {
            let tp = unsafe { *(timeout as *const TimeSpec) };
            let timeout = Duration::from(tp);
            info!("[do_futex_wait] timeout = {:?}", timeout);
            if let Err(Errno::ETIMEDOUT) = TimeoutFuture::new(futex_future, timeout).await {
                // info!("[do_futex_wait] time out.");
                // let pid = task.get_pid();
                // let mut binding = task.futex_list.lock();
                // if binding.check_is_inqueue(key, pid) {
                //     binding.remove(key, pid);
                //     return Err(Errno::EINVAL);
                // }
            };
        }
    }

    if task.sig_pending.lock().has_expected(wake_up_sig).0 {
        // 这里需要判断是否是被信号唤醒的
        info!("[do_futex_wait] wake up by signal");
        task.futex_list.lock().remove(key, task.get_pid());
        return Err(Errno::EINTR);
    }

    return Ok(0);
}

fn do_futex_wake(uaddr: usize, wake_n: u32, bitset: u32, key: FutexHashKey) -> SysResult<usize> {
    if bitset == 0 {
        return Err(Errno::EINVAL);
    }

    let task = current_task().unwrap();
    return Ok(task.futex_list.lock().to_wake(key, bitset, wake_n));
}

/// 主要作用是处理线程异常退出（如被强制终止或崩溃）时遗留的互斥锁（futex）问题，避免其他线程因无法获取锁而永久阻塞
/// set_ robust_ list系统调用就是为了解决这个问题而引入的。通过调用set robust list系统调用，
/// 进程可以将2个地址空间中的一块内存空间设置为robust列表,用于存储线
/// 程的信息。在线程被中断时，系统会将线程的相关信息保存在robust列表中，等线程重新启动时再从列表中读取信息，以便继续执行。
pub fn sys_set_robust_list(head: usize, len: usize) -> SysResult<usize> {
    info!("[sys_set_robust_list] start");
    if len != ROBUST_LIST_HEAD_SIZE {
        info!("robust list len invalid");
        return Err(Errno::EINVAL);
    }

    let task = current_task().unwrap();
    // 记录head
    task.robust_list.lock().head = head;
    return Ok(0);
}

/// The get_robust_list() system call returns the head of the robust
/// futex list of the thread whose thread ID is specified in pid.  If
/// pid is 0, the head of the list for the calling thread is returned.
/// The list head is stored in the location pointed to by head_ptr.
/// The size of the object pointed to by **head_ptr is stored in
/// sizep.
pub fn sys_get_robust_list(pid: usize, head: usize, sizep: usize) -> SysResult<usize> {
    info!("[sys_get_robust_list] start");
    let task = match pid {
        0 => current_task().unwrap(),
        p if pid > 0 => get_task_by_pid(p).ok_or(Errno::ESRCH)?,
        _ => return Err(Errno::EINVAL),
    };

    let token = task.get_user_token();
    let data = task.robust_list.lock().head;
    unsafe { core::ptr::write(head as *mut usize, data) };
    unsafe { core::ptr::write(sizep as *mut usize, ROBUST_LIST_HEAD_SIZE) };

    Ok(0)
}
