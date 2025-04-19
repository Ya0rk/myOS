// #![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use log::info;
use lwext4_rust::bindings::O_WRONLY;
use crate::{config::RLIMIT_NOFILE, fs::{FileTrait, OpenFlags, Stdin, Stdout}, mm::memory_space::{MmapFlags, MmapProt}, utils::{Errno, SysResult}};

#[derive(Clone)]
pub struct FdTable {
    pub table: Vec<FdInfo>, // 将fd作为下标idx
}

#[derive(Clone)]
pub struct FdInfo {
    pub file: Option<Arc<dyn FileTrait + Send + Sync>>,
    pub flags: OpenFlags,
}

impl FdInfo {
    pub fn new(fd: Arc<dyn FileTrait + Send + Sync>, flags: OpenFlags) -> Self {
        FdInfo {
            file: Some(fd),
            flags,
        }
    }

    pub fn new_bare() -> Self {
        FdInfo {
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

    pub fn check_mmap_valid(&self, flags:MmapFlags, prot: MmapProt) -> SysResult {
        if self.flags.contains(OpenFlags::O_WRONLY) {
            return Err(Errno::EACCES);
        }
        if flags.contains(MmapFlags::MAP_SHARED) && !self.flags.writable() && prot.contains(MmapProt::PROT_WRITE) {
            return Err(Errno::EACCES);
        }
        Ok(())

    }
}

impl FdTable {
    pub fn new() -> Self {
        // 自带三个文件描述符，分别是标准输入、标准输出、标准错误
        let stdin  = FdInfo::new(Arc::new(Stdin), OpenFlags::O_RDONLY);
        let stdout = FdInfo::new(Arc::new(Stdout), OpenFlags::O_WRONLY);
        let stderr = FdInfo::new(Arc::new(Stdout), OpenFlags::O_WRONLY);
        let mut fd_table = Vec::new();
        fd_table.push(stdin);
        fd_table.push(stdout);
        fd_table.push(stderr);
        FdTable {
            table: fd_table
        }
    }

    // 在task.exec中调用
    pub fn close_on_exec(&mut self) {
        for (fd, info) in self.table.iter_mut().enumerate() {
            if let Some(file) = &info.file {
                if info.flags.contains(OpenFlags::O_CLOEXEC) {
                    info.clear();
                }
            }
        }
    }

    /// 找到一个空位分配fd，返回fd的下标就是新fd
    pub fn alloc_fd(&mut self, fd: FdInfo) -> SysResult<usize> {
        // 先判断是否有没有使用的空闲fd， 用idx作为数组下标
        if let Some(valid_idx) = (0..self.table_len()).find(|idx| self.table[*idx].is_none()) {
            self.put_in(fd, valid_idx)?;
            Ok(valid_idx)
        } else {
            // 在最后加入
            // info!("before len = {}", self.table_len());
            let new_fd = self.table_len();
            self.put_in(fd, new_fd)?;
            // info!("after len = {}", self.table_len());
            Ok(new_fd)
        }
    }

    // 在指定位置加入Fd
    pub fn put_in(&mut self, fd: FdInfo, idx: usize) -> SysResult {
        if idx > RLIMIT_NOFILE {
            return Err(Errno::EMFILE);
        }
        if idx >= self.table_len() {
            self.table.resize(idx + 1, FdInfo::new_bare());
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
    pub fn get_file_by_fd(&self, idx: usize) -> SysResult<Option<Arc<dyn FileTrait + Send + Sync>>> {
        if idx >= self.table_len() {
            return  Err(Errno::EBADF);
        }
        Ok(self.table[idx].file.as_ref().map(|fd| fd.clone()))
    }

    pub fn get_fd(&self, idx: usize) -> SysResult<FdInfo> {
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