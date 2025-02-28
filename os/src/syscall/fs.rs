use alloc::string::String;
use log::info;

use crate::config::{AT_FDCWD, PATH_MAX, RLIMIT_NOFILE};
use crate::fs::{open, open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, Fd, FdTable};
use crate::utils::errtype::{Errno, SysResult};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> SysResult<usize> {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match inner.fd_table.get_file_by_fd(fd) {
        Ok(Some(file)) => {
            if !file.writable() {
                return Err(Errno::EPERM);
            }
            let file = file.clone();
            // release current task TCB manually to avoid multi-borrow
            drop(inner);
            Ok(file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as usize)
        }
        _ => Err(Errno::EBADCALL),
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> SysResult<usize> {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match inner.fd_table.get_file_by_fd(fd) {
        Ok(Some(file)) => {
            if !file.readable() {
                return Err(Errno::EPERM);
            }
            let file = file.clone();
            // release current task TCB manually to avoid multi-borrow
            drop(inner);
            Ok(file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as usize)
        }
        _ => Err(Errno::EBADCALL),
    }
}

/// 打开或创建一个文件：https://man7.org/linux/man-pages/man2/open.2.html
/// 
/// Success: 返回文件描述符; Fail: 返回-1
pub fn sys_openat(fd: isize, path: *const u8, flags: u32, _mode: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    let token = inner.get_user_token();
    let path = translated_str(token, path);
    let flags = OpenFlags::from_bits(flags as i32).unwrap();

    // 如果是绝对路径就忽略fd
    if path.starts_with('/') {
        if let Some(inode) = open_file(path.as_str(), flags) {
            let fd = inner.fd_table.alloc_fd(Fd::new(inode, flags))? as usize;
            return Ok(fd);
        } else {
            return Err(Errno::EBADCALL);
        }
    }

    // 如果是相对路径
    // 以当前目录作为出发点 open 文件
    if fd == AT_FDCWD {
        let cwd = inner.get_current_path();
        if let Some(inode) = open(cwd.as_str(), path.as_str(), flags) {
            let fd = inner.fd_table.alloc_fd(Fd::new(inode, flags))? as usize;
            return Ok(fd);
        } else {
            return Err(Errno::EBADCALL);
        }
    }

    // 如果是相对路径，并且不是以当前目录作为出发点
    // 就以fd作为出发点 open 文件
    
    if fd < 0 || fd as usize > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    if let Some(inode) = inner.fd_table.get_file_by_fd(fd as usize)? {
        let other_cwd = inode.get_name();
        // 释放锁, 因为在open函数中会再次获取锁
        drop(inner);
        if let Some(inode) = open(other_cwd.as_str(), path.as_str(), flags) {
            let mut inner = task.inner_lock();
            let fd = inner.fd_table.alloc_fd(Fd::new(inode, flags))? as usize;
            if fd > RLIMIT_NOFILE {
                return Err(Errno::EMFILE);
            }
            return Ok(fd);
        } else {
            return Err(Errno::EBADCALL);
        }
    }

    Err(Errno::EBADCALL)
}

pub fn sys_close(fd: usize) -> SysResult<usize> {
    info!("start sys_close");
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::EBADF);
    }
    
    // 删除对应的fd
    inner.fd_table.remove(fd)?;
    Ok(0)
}

/// 获取当前工作目录： https://man7.org/linux/man-pages/man3/getcwd.3.html
///
/// Success: 返回当前工作目录的长度;  Fail: 返回-1
pub fn sys_getcwd(buf: *mut u8, size: usize) -> SysResult<usize> {
    if buf.is_null() || (!buf.is_null() && size == 0) {
        return Err(Errno::EINVAL);
    }

    let task =  current_task().unwrap();
    let task_inner = task.inner_lock();
    let token = task_inner.get_user_token();
    let cwd: String = task_inner.get_current_path();
    let length: usize = cwd.len();

    if length > PATH_MAX {
        return Err(Errno::ENAMETOOLONG);
    }
    if length + 1 > size {
        return Err(Errno::ERANGE);
    }

    drop(task_inner);
    // TODO: 检测当前cwd是不是被unlinked： ENOENT The current working directory has been unlinked.
    // end

    let mut user_buffer = UserBuffer::new(translated_byte_buffer(token, buf, size));
    user_buffer.write(cwd.as_bytes());

    Ok(length)
}

/// 创建一个现有文件描述符的副本：https://man7.org/linux/man-pages/man2/dup.2.html
/// 
/// Success: 返回新的文件描述符; Fail: 返回-1
pub fn sys_dup(oldfd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    if oldfd >= inner.fd_table_len() {
        return Err(Errno::EBADF);
    }

    let old_temp_fd = inner.get_fd(oldfd);
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.clear_close_on_exec(true);
    let new_fd = FdTable::alloc_fd(&mut inner.fd_table, new_temp_fd)?;
    drop(inner);
    if new_fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    Ok(new_fd)
}

/// 将一个现有的文件描述符oldfd复制到一个新的文件描述符newfd上，newfd是用户指定的：https://man7.org/linux/man-pages/man2/dup.2.html
/// dup2 和 dup3 都使用此函数处理，只是dup3可以设置flags，dup2 的 flag默认为0
/// 
/// Success: 返回新的文件描述符; Fail: 返回-1
pub fn sys_dup3(oldfd: usize, newfd: usize, flags: u32) -> SysResult<usize> {
    if oldfd == newfd {
        return Err(Errno::EINVAL);
    }

    // 判断flags是否合法
    let flag = OpenFlags::from_bits(flags as i32).unwrap();
    let cloexec = {
        match flag {
            flags if flags.is_empty() => Some(false),
            OpenFlags::O_CLOEXEC => Some(true),
            _ => None,
        }
    }.ok_or(Errno::EINVAL)?;

    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    
    if newfd > RLIMIT_NOFILE ||
        oldfd >= inner.fd_table_len() ||
        inner.fd_is_none(oldfd) 
    {
        return Err(Errno::EBADF);
    }

    let old_temp_fd = inner.get_fd(oldfd);
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.clear_close_on_exec(cloexec);
    // 将newfd 放到指定位置
    inner.fd_table.put_in(new_temp_fd, newfd)?;

    Ok(newfd)
}