use core::mem::size_of;
use crate::fs::{open_file, FileClass, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer};
use crate::signal::{SigDetails, SigMask, UContext};
use crate::sync::time::{CLOCK_BOOTTIME, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID};
use crate::sync::{get_waker, sleep_for, suspend_now, yield_now, TimeSpec, TimeVal, Tms};
use crate::syscall::ffi::{CloneFlags, Utsname, WaitOptions};
use crate::task::{
    add_proc_group_member, add_task, current_task, current_user_token, extract_proc_to_new_group, get_proc_num, get_task_by_pid, spawn_user_task
};
use crate::utils::{Errno, SysResult, RNG};
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use log::{debug, info};
use lwext4_rust::bindings::true_;
use zerocopy::IntoBytes;

use super::ffi::Sysinfo;

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
    let req = *translated_ref(current_user_token(), req as *const TimeSpec);
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

    if flag.contains(CloneFlags::CLONE_SETTLS) {
        child_trap_cx.set_tp(tls);
    }
    // 检查是否需要设置 parent_tid
    if flag.contains(CloneFlags::CLONE_PARENT_SETTID) {
        *translated_refmut(token, ptid as *mut u32) = new_pid as u32;
    }
    // 检查是否需要设置子进程的 set_child_tid,clear_child_tid

    // 当使用 clone() 系统调用并设置 CLONE_CHILD_SETTID 标志时，
    // set_child_tid 会被设置为传递给 clone() 的 ctid 参数的值。
    // 新线程启动时，会将其线程 ID 写入该地址
    if flag.contains(CloneFlags::CLONE_CHILD_SETTID) {
        new_task.set_child_settid(ctid);
        *translated_refmut(token, ctid as *mut u32) = new_pid as u32;
    }
    // 检查是否需要设置child_cleartid,在线程退出时会将指向的地址清零

    // 当使用 CLONE_CHILD_CLEARTID 标志时，
    // clear_child_tid 会被设置为传递给 clone() 的 ctid 参数的值。
    // 当一个线程终止且其 clear_child_tid 不为 NULL 时，内核会在该地址写入 0，
    if flag.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
        new_task.set_child_cleartid(ctid);
    }

    // 子进程不能使用父进程的栈，所以需要手动指定
    if child_stack != 0 {
        child_trap_cx.set_sp(child_stack);
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

pub async fn sys_execve(path: usize, argv: usize, env: usize) -> SysResult<usize> {
    // info!("[sys_exec] start");
    let token = current_user_token();
    let path = translated_str(token, path as *const u8);
    debug!("sys_exec: path = {:?}", path);
    let mut args: Vec<String> = Vec::new();
    if argv != 0 {
        let argv = translated_ref(token, argv as *const &[usize]);
        for str_addr in argv.iter() {
            let arg_entry = translated_str(token, *str_addr as *const u8);
            args.push(arg_entry);
        }
    }
    let mut envs: Vec<String> = Vec::new();
    if env != 0 {
        let env = translated_ref(token, env as *const &[usize]);
        for str_addr in env.iter() {
            let env_entry = translated_str(token, *str_addr as *const u8);
            envs.push(env_entry);
        }
    }
    
    if let Some(FileClass::File(file)) = open_file(path.as_str(), OpenFlags::O_RDONLY) {
        // let all_data = app_inode.file()?.metadata.inode.read_all().await?;
        
        let task: alloc::sync::Arc<crate::task::TaskControlBlock> = current_task().unwrap();
        task.execve(file, args, envs).await;
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
            // pid = -1: 等待任意子进程
            -1 => {
                info!("wait any child");
                locked_child.values().find(|task| task.is_zombie() ).cloned()// 这里过滤掉了自己
            }
            // pid > 0：等待进程id为pid的子进程
            p if p > 0 => {
                info!("wait target pid = {}", p);
                locked_child.values().find(|task| task.is_zombie() && p as usize == task.get_pid()).cloned()
            }
            // pid < -1: 等待进程组标识符与pid绝对值相等的所有子进程
            p if p < -1 => {
                locked_child.values().find(|task| task.get_pid() == p.abs() as usize).cloned()
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
            info!("[sys_wait4]: task {} find a child: pid = {}, exit_code = {}.", task.get_pid(), child_pid, exit_code);
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

/// set pointer to thread ID
pub fn sys_set_tid_address(tidptr: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    task.set_child_cleartid(tidptr);
    Ok(task.get_pid())
}

/// exit all threads in a process
pub fn sys_exit_group(exit_code: i32) -> SysResult<usize> {
    let task = current_task().unwrap();
    task.kill_all_thread();
    task.set_exit_code((exit_code & 0xFF) << 8);
    Ok(0)
}

pub fn sys_clock_settime(
    clock_id: usize,
    timespec: *const u8,
) -> SysResult<usize> {
    if timespec.is_null() {
        info!("[sys_clock_settime] timespec is null");
        return Err(Errno::EBADCALL);
    }
    Ok(0)
}

pub fn sys_clock_gettime(
    clock_id: usize,
    timespec: *const u8,
) -> SysResult<usize> {
    if timespec.is_null() {
        info!("[sys_clock_gettime] timespec is null");
        return Err(Errno::EBADCALL);
    }
    let tp = timespec as *mut TimeSpec;
    let time = match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => TimeSpec::new(),
        CLOCK_PROCESS_CPUTIME_ID => TimeSpec::process_cputime_now(),
        CLOCK_THREAD_CPUTIME_ID => TimeSpec::thread_cputime_now(),
        CLOCK_BOOTTIME => TimeSpec::boottime_now(),
        _ => return Err(Errno::EINVAL),
    };
    unsafe { tp.write_volatile(time) };
    Ok(0)
}

/// creates a session and sets the process group ID
/// 调用进程成为新会话的领头进程(session leader)
/// 调用进程成为新进程组的领头进程(process group leader)
/// 调用进程不再有控制终端(controlling terminal)
pub fn sys_setsid() -> SysResult<usize> {
    let task = current_task().unwrap();
    let pid = task.get_pid();   // task的pid
    let old_pgid = task.get_pgid(); // task现在所属的进程组
    if !task.is_leader() {
        // set the calling task to new process group
        let new_pgid = pid;
        task.set_pgid(new_pgid); // 设置进程组ID为pid
        extract_proc_to_new_group(old_pgid, new_pgid, pid); // 从原进程组中提取，放入一个新的进程组
    }
    Ok(pid) // 返回新进程组的ID
}

/// sets the PGID of the process specified by pid to pgid.
/// 
/// If pid is zero, then the process ID of the calling process is used.  
/// If pgid is zero, then the PGID of the process specified by pid is made the same
/// as  its  process ID.  If setpgid() is used to move a process from one process group to another (as is done by some shells when creating pipelines), 
/// both process groups must be part of the same session. 
/// In this case, the pgid specifies an existing process group to be joined and the session ID of that group must match the session ID of the joining process.
pub fn sys_setpgid(pid: usize, pgid: usize) -> SysResult<usize> {
    if (pgid as isize) < 0 {
        return Err(Errno::EINVAL);
    }
    let target_task = match pid {
        0 => current_task().unwrap(),
        _ => get_task_by_pid(pid).ok_or(Errno::ESRCH)?,
    };

    let old_pgid = target_task.get_pgid();
    let pid = target_task.get_pid();
    if pgid == 0{
        let new_pgid = pid;
        target_task.set_pgid(pid);
        extract_proc_to_new_group(old_pgid, new_pgid, pid);
    } else {
        target_task.set_pgid(pgid);
        extract_proc_to_new_group(old_pgid, pgid, pid);
    }
    Ok(0)
}

/// sigreturn() is a system call that is used to restore the state of a process after it has been interrupted by a signal.
/// when a signal handler finished executing, it can call sigreturn() to restore the process's state to what it was before the signal was received.
/// 用于从信号处理函数返回到用户程序被中断的位置
pub fn sys_sigreturn() -> SysResult<usize> {
    let task = current_task().unwrap();
    let ucontext = task.get_ucontext() as *const UContext;
    let ucontext = unsafe { core::ptr::read(ucontext) };
    let sig_stack = ucontext.uc_stack;
    let sig_mask = ucontext.uc_sigmask;
    let trap_cx = task.get_trap_cx_mut();
    let sepc = ucontext.get_sepc();
    trap_cx.set_sepc(sepc); // 恢复sepc
    trap_cx.user_x = ucontext.get_userx(); // 恢复寄存器
    task.set_blocked(sig_mask); // 恢复信号屏蔽字
    // 恢复信号栈
    if sig_stack.ss_size != 0 {
        unsafe { *task.sig_stack.get() = Some(sig_stack) };
    }
    let a0 = trap_cx.user_x[10];
    Ok(a0)
}

/// return system information
pub fn sys_sysinfo(sysinfo: *const u8) -> SysResult<usize> {
    if sysinfo.is_null() {
        return Err(Errno::EBADCALL);
    }
    let proc_num = get_proc_num();
    let bind = Sysinfo::new(proc_num);
    let sysinfo = translated_refmut(current_user_token(), sysinfo as *mut Sysinfo);
    unsafe { core::ptr::write(sysinfo, bind) };
    Ok(0)
}