// #![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use log::info;
use lwext4_rust::bindings::O_WRONLY;
use crate::{
    fs::{FileTrait, OpenFlags, Stdin, Stdout}, hal::config::RLIMIT_NOFILE, mm::memory_space::{MmapFlags, MmapProt}, net::Socket, syscall::RLimit64, utils::{Errno, SysResult}
};

use super::current_task;

#[derive(Clone)]
pub struct FdTable {
    pub table: Vec<FdInfo>, // 将fd作为下标idx
    pub rlimit: RLimit64,
}

#[derive(Clone)]
pub struct FdInfo {
    pub file: Option<Arc<dyn FileTrait>>,
    pub flags: OpenFlags,
}

impl FdInfo {
    pub fn new(fd: Arc<dyn FileTrait>, flags: OpenFlags) -> Self {
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

    pub fn off_Ocloexec(mut self, enable: bool) -> Self {
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
            table: fd_table,
            rlimit: RLimit64 { rlim_cur: RLIMIT_NOFILE, rlim_max: RLIMIT_NOFILE }
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

    /// 找到一个空位分配fd，返回数组下标就是新fd
    pub fn alloc_fd(&mut self, info: FdInfo) -> SysResult<usize> {
        // 先判断是否有没有使用的空闲fd
        match self.find_slot(0) {
            Some(valid_fd) => {
                self.put_in(info, valid_fd)?;
                return Ok(valid_fd);
            }
            None => {
                // 在最后加入
                let new_fd = self.table_len();
                self.put_in(info, new_fd)?;
                return Ok(new_fd);
            }
        }
    }

    /// 分配一个大于than的fd
    pub fn alloc_fd_than(&mut self, info: FdInfo, than: usize) -> SysResult<usize> {
        // 先判断是否有没有使用的空闲fd
        match self.find_slot(than) {
            Some(valid_fd) => {
                self.put_in(info, valid_fd)?;
                return Ok(valid_fd);
            }
            None => {
                // 在最后加入
                let new_fd = self.table_len();
                self.put_in(info, new_fd)?;
                return Ok(new_fd);
            }
        }
    }

    pub fn find_slot(&self, start: usize) -> Option<usize> {
        if let Some(valid_fd) = (start..self.table_len()).find(|idx| self.table[*idx].is_none()) {
            return Some(valid_fd);
        }
        None
    }

    // 在指定位置加入Fd
    pub fn put_in(&mut self, info: FdInfo, idx: usize) -> SysResult {
        if idx > self.rlimit.rlim_cur {
            return Err(Errno::EMFILE);
        }
        if idx >= self.table_len() {
            self.table.resize(idx + 1, FdInfo::new_bare());
        }
        self.table[idx] = info;
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
    pub fn get_file_by_fd(&self, idx: usize) -> SysResult<Option<Arc<dyn FileTrait>>> {
        if idx >= self.table_len() {
            info!("[getfilebyfd] fdtable len = {}", self.table_len());
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
        self.table.clear();
    }
}

/// 将一个socket加入到fd表中
pub fn sock_map_fd(socket: Arc<dyn FileTrait>, cloexec_enable: bool) -> SysResult<usize> {
    let fdInfo = FdInfo::new(socket, OpenFlags::O_RDWR);
    let new_info = fdInfo.off_Ocloexec(cloexec_enable);
    let task = current_task().expect("no current task");
    let fd = task.alloc_fd(new_info);
    Ok(fd)
}