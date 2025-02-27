use alloc::string::String;

use crate::config::{PATH_MAX, RLIMIT_NOFILE};
use crate::fs::{open_file, OpenFlags};
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

pub fn sys_open(path: *const u8, flags: u32) -> SysResult<usize> {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    if let Some(inode) = open_file(path.as_str(), flags) {
        let mut inner = task.inner_lock();
        let fd = FdTable::alloc_fd(&mut inner.fd_table, Fd::new(inode, flags)).unwrap();
        Ok(fd as usize)
    } else {
        Err(Errno::EBADCALL)
    }
}

pub fn sys_close(fd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::EBADF);
    }
    if inner.fd_table.table[fd].is_none() {
        return Err(Errno::EBADCALL);
    }
    
    // 删除对应的fd
    inner.fd_table.table.remove(fd);
    Ok(0)
}

/// 获取当前工作目录： https://man7.org/linux/man-pages/man3/getcwd.3.html
///
/// Success: 返回当前工作目录的长度;  Fail: 返回-1
pub fn sys_getcwd(buf: *mut u8, size: usize) -> SysResult<usize> {
    if buf.is_null() || (!buf.is_null() && size == 0) {
        return Err(Errno::EINVAL);
    }

    let token = current_user_token();
    let task =  current_task().unwrap();
    let task_inner = task.inner_lock();
    let cwd: String = task_inner.get_current_path();
    let length: usize = cwd.len();

    if length > PATH_MAX {
        return Err(Errno::ENAMETOOLONG);
    }
    if length + 1 > size {
        return Err(Errno::ERANGE);
    }

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

    let old_temp_fd = inner.fd_table.get_by_fd(oldfd)?;
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.clear_close_on_exec(true);
    let new_fd = FdTable::alloc_fd(&mut inner.fd_table, new_temp_fd)?;

    if new_fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    Ok(new_fd)
}