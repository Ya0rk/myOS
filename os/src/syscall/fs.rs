use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, Fd, FdTable};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table.get_file_by_fd(fd) {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table.get_file_by_fd(fd) {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_lock();
        let fd = FdTable::alloc_fd(&mut inner.fd_table, Fd::new(inode)).unwrap();
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock();
    if fd >= inner.fd_table_len() {
        return -1;
    }
    if inner.fd_table.table[fd].is_none() {
        return -1;
    }
    // Todo 系统调用返回值修改为SysResult时， 后面修改为 remove函数
    inner.fd_table.table[fd].0.take();
    0
}