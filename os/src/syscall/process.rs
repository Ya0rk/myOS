use core::mem::size_of;
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer};
use crate::signal::{SigDetails, SigMask};
use crate::sync::{sleep_for, suspend_now, yield_now, TimeSepc, TimeVal, Tms};
use crate::syscall::ffi::{CloneFlags, Utsname, WaitOptions};
use crate::task::{
    add_task, current_task, current_user_token, spawn_user_task
};
use crate::utils::{Errno, SysResult, RNG};
use log::{debug, info};
use lwext4_rust::bindings::true_;
use zerocopy::IntoBytes;

// use super::ffi::Utsname;

pub fn sys_exit(exit_code: i32) -> SysResult<usize> {
    let task = current_task().unwrap();
    task.set_zombie();

    if task.is_leader(){
        info!("[sys_exit] task is leader, pid = {}, exit_code = {}", task.get_pid(), exit_code);
        task.set_exit_code((exit_code & 0xFF) << 8);
    }
    Ok(0)
}

pub async fn sys_nanosleep(req: usize, _rem: usize) -> SysResult<usize> {
    let req = *translated_ref(current_user_token(), req as *const TimeSepc);
    if !req.check_valid() {
        // info!("req = {}", req);
        return Err(Errno::EINVAL);
    }

    sleep_for(req).await;
    Ok(0)
}

pub async fn sys_yield() -> SysResult<usize> {
    info!("[sys_yield] start");
    yield_now().await;
    Ok(0)
}

/// 功能：获取进程时间；
/// 
/// 输入：tms结构体指针，用于获取保存当前进程的运行时间数据；
/// 
/// 返回值：成功返回已经过去的滴答数，失败返回-1;
pub fn sys_times(tms: *const u8) -> SysResult<usize> {
    if tms.is_null() {
        return Err(Errno::EBADCALL);
    }
    let bind = Tms::new();
    let time = bind.as_bytes();
    let mut buffer = UserBuffer::new(translated_byte_buffer(current_user_token(), tms, size_of::<Tms>()));
    buffer.write(time);
    Ok(0)
}

/// 功能：获取时间；
/// 
/// 输入： timespec结构体指针用于获得时间值；
/// 
/// 返回值：成功返回0，失败返回-1;
pub fn sys_gettimeofday(tv: *const u8, _tz: *const u8) -> SysResult<usize> {
    if tv.is_null() {
        return Err(Errno::EBADCALL);
    }
    let binding = TimeVal::new();
    let timeval = binding.as_bytes();
    let mut buffer = UserBuffer::new(translated_byte_buffer(current_user_token(), tv, size_of::<TimeVal>()));
    buffer.write(timeval);
    Ok(0)
}

/// 功能：打印系统信息；https://man7.org/linux/man-pages/man2/uname.2.html
/// 
/// 输入：utsname结构体指针用于获得系统信息数据；
/// 
/// 返回值：成功返回0，失败返回-1;
pub fn sys_uname(buf: *const u8) -> SysResult<usize> {
    debug!("sys_name start");
    if buf.is_null() {
        return Err(Errno::EBADCALL);
    }

    let bind = Utsname::new();
    let utsname = bind.as_bytes();
    let mut buffer = UserBuffer::new(
        translated_byte_buffer(
            current_user_token(), 
            buf, 
            size_of::<Utsname>()
    ));
    buffer.write(utsname);
    Ok(0)
}

pub fn sys_getpid() -> SysResult<usize> {
    Ok(current_task().unwrap().get_pid() as usize)
}

pub fn sys_getppid() -> SysResult<usize> {
    Ok(current_task().unwrap().get_ppid() as usize)
}

pub fn sys_clone(
    flags: usize,
    child_stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
    ) -> SysResult<usize> {
    debug!("start sys_fork");
    let flag = CloneFlags::from_bits(flags as u32).unwrap();
    let current_task = current_task().unwrap();
    let token = current_task.get_user_token();
    let new_task = match flag.contains(CloneFlags::CLONE_THREAD) {
        true  => current_task.thread_fork(flag),
        false => current_task.process_fork(flag),
    };
    drop(current_task);

    let new_pid = new_task.get_pid();
    let child_trap_cx = new_task.get_trap_cx_mut();

    // 子进程不能使用父进程的栈，所以需要手动指定
    if child_stack != 0 {
        child_trap_cx.set_sp(child_stack);
    }
    if flag.contains(CloneFlags::CLONE_SETTLS) {
        child_trap_cx.set_tp(tls);
    }
    // 检查是否需要设置 parent_tid
    if flag.contains(CloneFlags::CLONE_PARENT_SETTID) {
        *translated_refmut(token, ptid as *mut u32) = new_pid as u32;
    }
    // 检查是否需要设置子进程的 set_child_tid,clear_child_tid
    if flag.contains(CloneFlags::CLONE_CHILD_SETTID) {
        *translated_refmut(token, ctid as *mut u32) = new_pid as u32;
    }
    // 检查是否需要设置child_cleartid,在线程退出时会将指向的地址清零
    if flag.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
        new_task.set_child_cleartid(ctid);
    }

    // 因为我们已经在trap_handler中增加了sepc，所以这里不需要再次增加
    // 只需要修改子进程返回值为0即可
    child_trap_cx.user_x[10] = 0;
    // 将子进程加入任务管理器，这里可以快速找到进程
    add_task(&new_task);
    spawn_user_task(new_task);
    // info!("[sys_fork] finished new pid = {}", new_pid);

    // 父进程返回子进程的pid
    Ok(new_pid as usize)
}

pub async fn sys_exec(path: usize) -> SysResult<usize> {
    // info!("[sys_exec] start");
    let token = current_user_token();
    let path = translated_str(token, path as *const u8);
    debug!("sys_exec: path = {:?}", path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::O_RDONLY) {
        let all_data = app_inode.file()?.metadata.inode.read_all().await?;
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        Ok(0)
    } else {
        Err(Errno::EBADCALL)
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
/// pid = -1: 等待任意子进程
/// pid = 0 : 等待与调用进程（父进程）同一个进程组的所有子进程
/// pid < -1: 等待进程组标识符与pid绝对值相等的所有子进程
/// pid > 0 ：等待进程id为pid的子进程
pub async fn sys_wait4(pid: isize, wstatus: usize, options: usize, _rusage: usize) -> SysResult<usize> {
    debug!("sys_wait4 start, pid = {}, options = {}", pid, options);
    // info!("[sys_wait4] start, pid = {}, options = {}", pid,options);
    let task = current_task().unwrap();
    let self_pid = task.get_pid();
    if task.children.lock().is_empty() {
        info!("task pid = {}, has no child.", task.get_pid());
        return Err(Errno::ECHILD);
    }

    let op = WaitOptions::from_bits(options as i32).unwrap();

    // 缩小 locked_child 的作用域
    let target_task = {
        let locked_child = task.children.lock().clone();
        match pid {
            -1 => {
                locked_child.values().find(|task| task.is_zombie() && task.get_pid() != self_pid).cloned()// 这里过滤掉了自己
            }
            p if p > 0 => {
                locked_child.values().find(|task| task.is_zombie() && p as usize == task.get_pid()).cloned()
            }
            _ => unimplemented!(),
        }
    };

    match target_task {
        Some(zombie_child) => {
            info!("[sys_wait4] find a target zombie child task pid = {}.", zombie_child.get_pid());
            let zombie_pid = zombie_child.get_pid();
            let exit_code = zombie_child.get_exit_code();
            task.do_wait4(zombie_pid, wstatus as *mut i32, exit_code);
            return Ok(zombie_pid);
        }
        None => {
            info!("[sys_wait4] current task pid = {}", task.get_pid());
            if op.contains(WaitOptions::WNOHANG) {
                return Ok(0)
            }
            // 如果等待的进程还不是zombie，那么本进程进行await，
            // 直到等待的进程do_exit然后发送SIGCHLD信号唤醒自己
            let (child_pid, _status, exit_code) = loop {
                task.set_wake_up_signal(!*task.get_blocked() | SigMask::SIGCHLD);
                suspend_now().await;
                // 在pending队列中取出希望的信号，也就是子进程结束后发送给父进程的信号
                match task.sig_pending.lock().take_expected_one(SigMask::SIGCHLD) {
                    Some(sig_info) => {
                        if let SigDetails::Chld { 
                            pid: find_pid, 
                            status, 
                            exit_code 
                        } = sig_info.sifields
                        {
                            match pid {
                                -1 => break (find_pid, status, exit_code),
                                p if p > 0 => {
                                    if find_pid == p as usize {
                                        break (find_pid, status, exit_code);
                                    }
                                }
                                _ => unimplemented!(),
                            }
                        }
                    }
                    None => return Err(Errno::EINTR),
                }
            };
            info!("[sys_wait4]: find a child: pid = {}, exit_code = {}.", child_pid, exit_code);
            task.do_wait4(child_pid, wstatus as *mut i32, exit_code);
            // info!("[do_wait4] child_pid = {}", child_pid);
            return Ok(child_pid);
        }
    }
}


pub fn sys_getrandom(
    buf: *const u8,
    buflen: usize,
    _flags: usize,
) -> SysResult<usize> {
    let token = current_user_token();
    let buffer = UserBuffer::new(
        translated_byte_buffer(
            token,
            buf,
            buflen
    ));
    Ok(RNG.lock().fill_buf(buffer))
}