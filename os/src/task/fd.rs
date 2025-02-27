// #![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use crate::{fs::{File, Stdin, Stdout}, utils::errtype::{SysResult, Errno}};

#[derive(Clone)]
pub struct FdTable {
    pub table: Vec<Fd>,
}

#[derive(Clone)]
pub struct Fd(pub Option<Arc<dyn File + Send + Sync>>);

impl Fd {
    pub fn new(fd: Arc<dyn File + Send + Sync>) -> Self {
        Fd(Some(fd))
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl FdTable {
    pub fn new() -> Self {
        // 自带三个文件描述符，分别是标准输入、标准输出、标准错误
        let stdin  = Fd::new(Arc::new(Stdin));
        let stdout = Fd::new(Arc::new(Stdout));
        let stderr = Fd::new(Arc::new(Stdout));
        let mut fd_table = Vec::new();
        fd_table.push(stdin);
        fd_table.push(stdout);
        fd_table.push(stderr);
        FdTable {
            table: fd_table
        }
    }

    pub fn alloc_fd(&mut self, fd: Fd) -> SysResult<usize> {
        // 先判断是否有没有使用的空闲fd， 用idx作为数组下标
        if let Some(valid_idx) = (0..self.table_len()).find(|idx| self.table[*idx].is_none()) {
            self.put_in(fd, valid_idx)?;
            Ok(valid_idx)
        } else {
            // 在最后加入
            let new_fd = self.table_len();
            self.put_in(fd, new_fd)?;
            Ok(new_fd)
        }
    }

    // 在指定位置加入Fd
    pub fn put_in(&mut self, fd: Fd, idx: usize) -> SysResult {
        if idx >= self.table_len() {
            self.table.push(Fd(None));
        }
        self.table[idx] = fd;
        Ok(())
    }

    pub fn remove(&mut self, fd: usize) -> SysResult {
        if fd >= self.table_len() {
            return Err(Errno::EBADF);
        }
        self.table[fd].0.take();
        Ok(())
    }

    pub fn table_len(&self) -> usize {
        self.table.len()
    }

    pub fn get_file_by_fd(&self, idx: usize) -> SysResult<Option<Arc<dyn File + Send + Sync>>> {
        if idx >= self.table_len() {
            return  Err(Errno::EBADF);
        }
        Ok(self.table[idx].0.as_ref().map(|fd| fd.clone()))
    }
}