use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, Fd, FdTable};
use crate::utils::errtype::{Errno, SysResult};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> SysResult<usize> {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::ErrBADF);
    }
    match inner.fd_table.get_file_by_fd(fd) {
        Ok(Some(file)) => {
            if !file.writable() {
                return Err(Errno::ErrPERM);
            }
            let file = file.clone();
            // release current task TCB manually to avoid multi-borrow
            drop(inner);
            Ok(file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as usize)
        }
        _ => Err(Errno::ErrBADCALL),
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> SysResult<usize> {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::ErrBADF);
    }
    match inner.fd_table.get_file_by_fd(fd) {
        Ok(Some(file)) => {
            if !file.readable() {
                return Err(Errno::ErrPERM);
            }
            let file = file.clone();
            // release current task TCB manually to avoid multi-borrow
            drop(inner);
            Ok(file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as usize)
        }
        _ => Err(Errno::ErrBADCALL),
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> SysResult<usize> {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_lock();
        let fd = FdTable::alloc_fd(&mut inner.fd_table, Fd::new(inode)).unwrap();
        Ok(fd as usize)
    } else {
        Err(Errno::ErrBADCALL)
    }
}

pub fn sys_close(fd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return Err(Errno::ErrBADF);
    }
    if inner.fd_table.table[fd].is_none() {
        return Err(Errno::ErrBADCALL);
    }
    // Todo 系统调用返回值修改为SysResult时， 后面修改为 remove函数
    inner.fd_table.table[fd].0.take();
    Ok(0)
}