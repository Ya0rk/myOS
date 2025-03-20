// #![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use crate::{config::RLIMIT_NOFILE, fs::{File, OpenFlags, Stdin, Stdout}, utils::{Errno, SysResult}};

#[derive(Clone)]
pub struct FdTable {
    pub table: Vec<Fd>, // 将fd作为下标idx
}

#[derive(Clone)]
pub struct Fd {
    pub file: Option<Arc<dyn File + Send + Sync>>,
    pub flags: OpenFlags,
}

impl Fd {
    pub fn new(fd: Arc<dyn File + Send + Sync>, flags: OpenFlags) -> Self {
        Fd {
            file: Some(fd),
            flags,
        }
    }

    pub fn new_bare() -> Self {
        Fd {
            file: None,
            flags: OpenFlags::empty(),
        }
    }

    pub fn clear(&mut self) {
        self.file = None;
        self.flags = OpenFlags::empty();
    }

    pub fn is_none(&self) -> bool {
        self.file.is_none() && self.flags.is_empty()
    }

    pub fn set_close_on_exec(mut self, enable: bool) -> Self {
        if enable {
            self.flags.remove(OpenFlags::O_CLOEXEC);
        } else {
            self.flags.insert(OpenFlags::O_CLOEXEC);
        }
        self
    }
}

impl FdTable {
    pub fn new() -> Self {
        // 自带三个文件描述符，分别是标准输入、标准输出、标准错误
        let stdin  = Fd::new(Arc::new(Stdin), OpenFlags::O_RDONLY);
        let stdout = Fd::new(Arc::new(Stdout), OpenFlags::O_WRONLY);
        let stderr = Fd::new(Arc::new(Stdout), OpenFlags::O_WRONLY);
        let mut fd_table = Vec::new();
        fd_table.push(stdin);
        fd_table.push(stdout);
        fd_table.push(stderr);
        FdTable {
            table: fd_table
        }
    }

    /// 找到一个空位分配fd，返回fd的下标就是新fd
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
        if idx > RLIMIT_NOFILE {
            return Err(Errno::EMFILE);
        }
        if idx >= self.table_len() {
            self.table.resize(idx + 1, Fd::new_bare());
        }
        self.table[idx] = fd;
        Ok(())
    }

    pub fn remove(&mut self, fd: usize) -> SysResult {
        if fd >= self.table_len() || self.table[fd].is_none() {
            return Err(Errno::EBADF);
        }
        self.table[fd].clear();
        Ok(())
    }

    pub fn table_len(&self) -> usize {
        self.table.len()
    }

    /// 通过fd获取文件
    pub fn get_file_by_fd(&self, idx: usize) -> SysResult<Option<Arc<dyn File + Send + Sync>>> {
        if idx >= self.table_len() {
            return  Err(Errno::EBADF);
        }
        Ok(self.table[idx].file.as_ref().map(|fd| fd.clone()))
    }

    pub fn get_fd(&self, idx: usize) -> SysResult<Fd> {
        if idx >= self.table_len() {
            return Err(Errno::EBADF);
        }
        Ok(self.table[idx].clone())
    }

    pub fn clear(&mut self) {
        for fd in &mut self.table {
            fd.clear();
        }
    }
}