use core::mem::size_of;
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer};
use crate::sync::timer::{sleep_for, TimeSepc, TimeVal};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::utils::errtype::{Errno, SysResult};
use alloc::sync::Arc;
use log::info;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_nanosleep(req: *const u8, _rem: *const u8) -> SysResult<usize> {
    let req = *translated_ref(current_user_token(), req as *const TimeSepc);
    if !req.check_valid() {
        return Err(Errno::EINVAL);
    }

    sleep_for(req);
    Ok(0)
}

pub fn sys_yield() -> SysResult<usize> {
    suspend_current_and_run_next();
    Ok(0)
}

pub fn sys_gettimeofday(tv: *const u8, _tz: *const u8) -> SysResult<usize> {
    let binding = TimeVal::new();
    let timeval = binding.as_bytes();
    let mut buffer = UserBuffer::new(translated_byte_buffer(current_user_token(), tv, size_of::<TimeVal>()));
    buffer.write(timeval);
    Ok(0)
}

pub fn sys_getpid() -> SysResult<usize> {
    Ok(current_task().unwrap().get_pid() as usize)
}

pub fn sys_getppid() -> SysResult<usize> {
    Ok(current_task().unwrap().get_ppid() as usize)
}

pub fn sys_fork() -> SysResult<usize> {
    info!("start sys_fork");
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let child_trap_cx = new_task.inner_lock().get_trap_cx();
    // 因为我们已经在trap_handler中增加了sepc，所以这里不需要再次增加
    // 只需要修改子进程返回值为0即可
    child_trap_cx.user_x[10] = 0;
    // 将子进程加入调度器，等待被调度
    add_task(new_task);
    // 父进程返回子进程的pid
    Ok(new_pid as usize)
}

pub fn sys_exec(path: *const u8) -> SysResult<usize> {
    let token = current_user_token();
    let path = translated_str(token, path);
    println!("sys_exec: path = {:?}", path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::O_RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        Ok(0)
    } else {
        Err(Errno::EBADCALL)
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> SysResult<usize> {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_lock();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return Err(Errno::NOPID);
        // ---- release current PCB
    }

    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_lock().exit_code;
        // ++++ release child PCB
        if !exit_code_ptr.is_null() {
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        }
        Ok(found_pid as usize)
    } else {
        Err(Errno::HAVEPID)
    }
    // ---- release current PCB lock automatically
}
