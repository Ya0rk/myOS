use core::mem::size_of;
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer};
use crate::hal::arch::timer::{sleep_for, TimeSepc, TimeVal, Tms};
use crate::syscall::ffi::Utsname;
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::utils::{Errno, SysResult};
use alloc::sync::Arc;
use log::debug;
use zerocopy::IntoBytes;

// use super::ffi::Utsname;

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

pub fn sys_clone() -> SysResult<usize> {
    debug!("start sys_fork");
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
    debug!("new pid is : {}", new_pid);
    // 父进程返回子进程的pid
    Ok(new_pid as usize)
}

pub fn sys_exec(path: *const u8) -> SysResult<usize> {
    let token = current_user_token();
    let path = translated_str(token, path);
    debug!("sys_exec: path = {:?}", path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::O_RDONLY) {
        let all_data = app_inode.file()?.inode.read_all()?;
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        Ok(0)
    } else {
        Err(Errno::EBADCALL)
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_wait4(pid: isize, exit_code_ptr: *mut i32, options: usize, _rusage: usize) -> SysResult<usize> {
    debug!("sys_wait4 start, pid = {}, options = {}", pid, options);
    let task = current_task().unwrap();
    
    loop {
        // 获取当前任务的内部锁
        let mut inner = task.inner_lock();

        // 快速查看是否存在符合条件的子进程
        if !inner
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.getpid())
        {
            return Err(Errno::NOPID);
        }

        // 查找僵尸子进程
        let zombie_child = inner.children.iter().enumerate().find_map(|(idx, p)| {
            //检查是否为僵尸进程且符合 PID 条件
            if p.inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid()) {
                Some(idx)
            } else {
                None
            }
        });

        if let Some(idx) = zombie_child {
            // 移除子进程
            let removed_child = inner.children.remove(idx);
            // 确认子进程的引用计数为 1
            assert_eq!(Arc::strong_count(&removed_child), 1);

            // 获取子进程的 PID 和退出状态
            let found_pid = removed_child.getpid();
            let exit_code = removed_child.inner_lock().exit_code;

            // 将退出状态写入用户提供的指针
            if !exit_code_ptr.is_null() {
                *translated_refmut(inner.memory_set.token(), exit_code_ptr) = (exit_code & 0xff) << 8;
            }
            return Ok(found_pid as usize);
        } else {
            // 未找到僵尸子进程，释放锁并挂起当前任务
            drop(inner); // 避免死锁
            suspend_current_and_run_next();
        }
    }
    // ---- release current PCB lock automatically
}
