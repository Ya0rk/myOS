use core::{cmp::min, fmt::Display, intrinsics::unlikely};

// #![allow(unused)]
use super::current_task;
use crate::{
    fs::{open, socketfs::{socketfile::SocketFile, socketinode::SocketInode}, FileTrait, InodeTrait, Kstat, OpenFlags, Page, RenameFlags}, hal::config::RLIMIT_NOFILE, mm::memory_space::{MmapFlags, MmapProt}, net::{Socket, PORT_FD_MANAMER}, sync::time_duration, syscall::RLimit64, utils::{Errno, SysResult}
};
use alloc::{collections::binary_heap::BinaryHeap, format, string::String, sync::Arc, vec::Vec};
use log::info;
use lwext4_rust::bindings::O_WRONLY;

const BITS_PER_BLOCK: usize = 64; // 每个位图块64位
#[derive(Clone)]
pub struct FdTable {
    pub table: Vec<FdInfo>, // 将fd作为下标idx
    pub rlimit: RLimit64,
    free_bitmap: Vec<u64>,   // 空闲FD位图 (1表示空闲, 0表示已使用)
    next_free: usize,        // 快速查找起点
    freed_stack: Vec<usize>, // 保存最近释放的FD缓存
}

#[derive(Clone)]
pub struct FdInfo {
    pub file: Option<Arc<dyn FileTrait>>,
    pub flags: OpenFlags,
}

impl FdInfo {
    pub fn new(file: Arc<dyn FileTrait>, flags: OpenFlags) -> Self {
        FdInfo {
            file: Some(file),
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

    pub fn check_mmap_valid(&self, flags: MmapFlags, prot: MmapProt) -> SysResult {
        if self.flags.contains(OpenFlags::O_WRONLY) {
            return Err(Errno::EACCES);
        }
        if flags.contains(MmapFlags::MAP_SHARED)
            && !self.flags.writable()
            && prot.contains(MmapProt::PROT_WRITE)
        {
            return Err(Errno::EACCES);
        }
        Ok(())
    }
}

impl Display for FdTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut msgs = String::from("FD TABLE:");
        for (i, item) in self.table.iter().enumerate() {
            if let Some(file) = &item.file {
                let msg = format!("\n   {}: {}", i, file.abspath());
                msgs.push_str(&msg);
            }
        }
        write!(f, "{}", msgs)
    }
}

impl FdTable {
    // 内部方法：释放FD槽位
    fn free_fd_slot(&mut self, fd: usize) {
        // 只缓存非末尾的FD (末尾FD在扩展时会自动重用)
        if fd < self.table_len() - 1 {
            // info!("push to freed stask, fd = {}", fd);
            self.freed_stack.push(fd);
        }

        // 更新位图 (标记为空闲)
        self.update_bitmap(fd, true);

        // 更新快速查找起点
        if fd < self.next_free {
            self.next_free = fd;
        }
    }
    // 位图操作：更新指定FD的状态
    fn update_bitmap(&mut self, fd: usize, is_free: bool) {
        let block_idx = fd / BITS_PER_BLOCK;
        let bit_offset = fd % BITS_PER_BLOCK;
        let mask = 1 << bit_offset;

        if block_idx < self.free_bitmap.len() {
            if is_free {
                // 设置位 (标记为空闲)
                self.free_bitmap[block_idx] |= mask;
            } else {
                // 清除位 (标记为已使用)
                self.free_bitmap[block_idx] &= !mask;
            }
        }
    }
    // 使用位图查找空闲FD
    fn find_free_by_bitmap(&mut self) -> Option<usize> {
        // 从next_free开始查找
        let start_block = self.next_free / BITS_PER_BLOCK;

        for block_idx in start_block..self.free_bitmap.len() {
            let bits = self.free_bitmap[block_idx];
            if bits == 0 {
                continue; // 该块无空闲位
            }

            // 找到第一个空闲位
            let offset = bits.trailing_zeros() as usize;
            let fd = block_idx * BITS_PER_BLOCK + offset;

            // 确保FD在表范围内
            if fd < self.table_len() {
                self.next_free = fd + 1; // 更新查找起点
                return Some(fd);
            }
        }

        // 没有找到空闲位
        self.next_free = self.table_len(); // 重置查找起点
        None
    }

    // 确保位图足够大以覆盖指定FD
    fn ensure_bitmap_size(&mut self, fd: usize) {
        let blocks_needed = (fd / BITS_PER_BLOCK) + 1;
        while self.free_bitmap.len() < blocks_needed {
            // 新块初始化为全空闲 (所有位为1)
            self.free_bitmap.push(u64::MAX);
        }
    }
}

impl FdTable {
    pub fn new() -> Self {
        let mut stdin;
        let mut stdout;
        let mut stderr;
        // 自带三个文件描述符，分别是标准输入、标准输出、标准错误
        #[cfg(any(feature = "2k1000la"))]
        {
            use crate::fs::DevTty;
            stdin = FdInfo::new(Arc::new(DevTty::new_in()), OpenFlags::O_RDONLY);
            stdout = FdInfo::new(Arc::new(DevTty::new_out()), OpenFlags::O_WRONLY);
            stderr = FdInfo::new(Arc::new(DevTty::new_out()), OpenFlags::O_WRONLY);
        }
        #[cfg(any(feature = "board_qemu", feature = "vf2"))]
        {
            use crate::fs::CharDev;
            stdin = FdInfo::new(Arc::new(CharDev::new_in()), OpenFlags::O_RDONLY);
            stdout = FdInfo::new(Arc::new(CharDev::new_out()), OpenFlags::O_WRONLY);
            stderr = FdInfo::new(Arc::new(CharDev::new_out()), OpenFlags::O_WRONLY);
        }
        let mut fd_table = Vec::new();
        fd_table.push(stdin);
        fd_table.push(stdout);
        fd_table.push(stderr);

        // 初始化位图：0,1,2 已使用 (位设置为0)，其余位空闲 (设置为1)
        let mut free_bitmap = Vec::new();
        free_bitmap.push(0xFFFF_FFFF_FFFF_FFF8); // 低3位为0，其余为1

        FdTable {
            table: fd_table,
            rlimit: RLimit64 {
                rlim_cur: RLIMIT_NOFILE,
                rlim_max: RLIMIT_NOFILE,
            },
            free_bitmap,
            next_free: 0,
            freed_stack: Vec::new(),
        }
    }

    // 在task.exec中调用
    pub fn close_on_exec(&mut self) {
        let mut to_free = Vec::new();
        for (fd, info) in self.table.iter_mut().enumerate() {
            if let Some(file) = &info.file {
                if info.flags.contains(OpenFlags::O_CLOEXEC) {
                    to_free.push(fd);
                    // info.clear();
                }
            }
        }
        // 清理已关闭的文件描述符
        for fd in to_free {
            self.free_fd_slot(fd);
            self.table[fd].clear();
        }
    }

    /// 找到一个空位分配fd，返回数组下标就是新fd
    pub fn alloc_fd(&mut self, info: FdInfo) -> SysResult<usize> {
        // 1. 优先使用最近释放的缓存
        // info!("freed stask {:?}", self.freed_stack);
        if let Some(fd) = self.freed_stack.pop() {
            self.update_bitmap(fd, false); // 标记为已使用
            self.put_in(info, fd)?;
            // info!("from freed stask, fd = {}", fd);
            return Ok(fd);
        }

        // 2. 使用位图快速查找空闲FD
        if let Some(fd) = self.find_free_by_bitmap() {
            self.update_bitmap(fd, false); // 标记为已使用
            self.put_in(info, fd)?;
            // info!("from bitmap, fd = {}", fd);
            return Ok(fd);
        }

        // 3. 扩展表,没有空闲的
        let new_fd = self.table_len();
        if new_fd >= self.rlimit.rlim_cur as usize {
            return Err(Errno::EMFILE);
        }

        self.ensure_bitmap_size(new_fd);
        self.update_bitmap(new_fd, false); // 新FD标记为已使用
        self.put_in(info, new_fd)?;
        Ok(new_fd)

        // // 先判断是否有没有使用的空闲fd
        // match self.find_slot(0) {
        //     Some(valid_fd) => {
        //         self.put_in(info, valid_fd)?;
        //         return Ok(valid_fd);
        //     }
        //     None => {
        //         // 在最后加入
        //         let new_fd = self.table_len();
        //         // info!("newfd = {}, limit = {}", new_fd, self.rlimit.rlim_cur);
        //         self.put_in(info, new_fd)?;
        //         return Ok(new_fd);
        //     }
        // }
    }

    /// 分配一个大于than的fd
    pub fn alloc_fd_than(&mut self, info: FdInfo, than: usize) -> SysResult<usize> {
        // 先判断是否有没有使用的空闲fd
        match self.find_slot(than) {
            Some(valid_fd) => {
                self.ensure_bitmap_size(valid_fd);
                self.update_bitmap(valid_fd, false); // 标记为已使用
                self.freed_stack.retain(|&x| x != valid_fd);
                self.put_in(info, valid_fd)?;
                return Ok(valid_fd);
            }
            None => {
                // 在最后加入
                let new_fd = self.table_len();
                self.ensure_bitmap_size(new_fd);
                self.update_bitmap(new_fd, false);
                self.freed_stack.retain(|&x| x != new_fd);
                self.put_in(info, new_fd)?;
                return Ok(new_fd);
            }
        }
    }

    pub fn find_slot(&self, start: usize) -> Option<usize> {
        if start >= self.table_len() {
            return Some(start);
        }
        let start = min(self.next_free, start);
        if let Some(valid_fd) = (start..self.table_len()).find(|idx| self.table[*idx].is_none()) {
            return Some(valid_fd);
        }
        None
    }

    // 在指定位置加入Fd
    pub fn put_in(&mut self, info: FdInfo, idx: usize) -> SysResult {
        if unlikely(idx >= self.rlimit.rlim_cur) {
            return Err(Errno::EMFILE);
        }
        if idx >= self.table_len() {
            self.table.resize(idx + 1, FdInfo::new_bare());
        }
        self.table[idx] = info;
        Ok(())
    }

    pub fn remove(&mut self, fd: usize) -> SysResult {
        if self.table[fd].is_none() {
            return Ok(());
        }
        if fd >= self.table_len() {
            return Err(Errno::EBADF);
        }
        let file = self.table[fd].file.take().unwrap();
        if file.metadata().inode.metadata()._type.is_socket() {
            let pid = current_task().unwrap().get_pid();
            PORT_FD_MANAMER.lock().remove_all_fds_by_pid_and_fd(pid, fd);
        }
        self.table[fd].clear();
        self.free_fd_slot(fd); // 释放fd槽位
        Ok(())
    }

    pub fn table_len(&self) -> usize {
        self.table.len()
    }

    /// 通过fd获取文件
    pub fn get_file_by_fd(&self, idx: usize) -> SysResult<Option<Arc<dyn FileTrait>>> {
        if idx >= self.table_len() {
            // info!("[getfilebyfd] fdtable len = {}", self.table_len());
            return Err(Errno::EBADF);
        }
        Ok(self.table[idx].file.as_ref().map(|fd| fd.clone()))
    }

    pub fn get_fdinfo(&self, idx: usize) -> SysResult<FdInfo> {
        if idx >= self.table_len() {
            return Err(Errno::EBADF);
        }
        Ok(self.table[idx].clone())
    }

    /// 获取fdinfo的可变引用，修改里面的数据
    pub fn get_mut_fdinfo(&mut self, idx: usize) -> SysResult<&mut FdInfo> {
        if idx >= self.table_len() {
            return Err(Errno::EBADF);
        }
        Ok(&mut self.table[idx])
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }
}

/// 将一个socket加入到fd表中
pub fn sock_map_fd(socket: Arc<dyn Socket>, cloexec_enable: bool, flags: OpenFlags) -> SysResult<usize> {
    // let mut flag = OpenFlags::O_RDWR; // 这里的flag基本没用
    let socketinode = Arc::new(SocketInode::new(socket));
    let socketfile = Arc::new(SocketFile::new(flags, socketinode));

    let fdInfo = FdInfo::new(socketfile, flags);
    // let new_info = fdInfo.off_Ocloexec(!cloexec_enable);
    let task = current_task().expect("no current task");
    let fd = task.alloc_fd(fdInfo)?;
    info!("socket map fd: new fd = {}, flags = {:?}", fd, flags);
    Ok(fd)
}

pub fn exchange_sock_fdinfo(oldfd: usize, newfd: usize) -> SysResult<()> {
    let task = current_task().unwrap();
    if unlikely(oldfd >= task.fd_table_len() || newfd >= task.fd_table_len()) {
        // info!("[exchange_sock_fdinfo] out of range: oldfd = {}, newfd = {}, fdtable len = {}",
        //     oldfd,
        //     newfd,
        //     task.fd_table_len()
        // );
        return Err(Errno::EBADF);
    }
    let mut fdtable = task.fd_table.lock();
    fdtable.table.swap(oldfd, newfd);
    let temp = fdtable.table[newfd].flags;
    fdtable.table[newfd].flags = fdtable.table[oldfd].flags;
    fdtable.table[oldfd].flags = temp;
    Ok(())
}

pub fn test_fd_performance() {
    use alloc::sync::Arc;
    use core::time::Duration;

    println!("Starting FD table performance tests...");
    let testfile = open("/aaa".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR)
        .unwrap();

    // 测试1: 顺序分配性能
    let test_sequential_allocation = |size: usize| -> (Duration, Duration) {
        let start = time_duration();
        let mut table = FdTable::new();
        table.rlimit.rlim_cur = size + 3; // 标准输入/输出/错误占3个

        for _ in 0..size {
            table
                .alloc_fd(FdInfo::new(testfile.clone(), OpenFlags::empty()))
                .unwrap();
        }

        (start, time_duration())
    };

    // 测试2: 随机释放再分配性能
    let test_random_reuse = |size: usize| -> (Duration, Duration) {
        let mut table = FdTable::new();
        table.rlimit.rlim_cur = size + 3;

        // 先分配所有FD
        let mut fds: Vec<usize> = (0..size)
            .map(|_| {
                table
                    .alloc_fd(FdInfo::new(testfile.clone(), OpenFlags::empty()))
                    .unwrap()
            })
            .collect();

        // 随机释放一半FD
        for i in (0..size).step_by(2) {
            table.remove(fds[i]).unwrap();
        }

        let start = time_duration();
        // 重新分配释放的FD
        for _ in 0..size / 2 {
            table
                .alloc_fd(FdInfo::new(testfile.clone(), OpenFlags::empty()))
                .unwrap();
        }

        (start, time_duration())
    };

    // 测试3: 高频周转性能
    let test_high_turnover = |cycles: usize, batch: usize| -> (Duration, Duration) {
        let mut table = FdTable::new();
        table.rlimit.rlim_cur = (batch * 2) + 3;

        let start = time_duration();
        for _ in 0..cycles {
            let mut fds = Vec::with_capacity(batch);
            // 分配一批FD
            for _ in 0..batch {
                fds.push(
                    table
                        .alloc_fd(FdInfo::new(testfile.clone(), OpenFlags::empty()))
                        .unwrap(),
                );
            }
            // 立即释放这批FD
            for fd in fds {
                table.remove(fd).unwrap();
            }
        }

        (start, time_duration())
    };

    // 测试4: 分配大于指定值的FD
    let test_alloc_than = |size: usize| -> (Duration, Duration) {
        let mut table = FdTable::new();
        table.rlimit.rlim_cur = size + 20;

        // 预先分配一些低FD
        for _ in 0..10 {
            table
                .alloc_fd(FdInfo::new(testfile.clone(), OpenFlags::empty()))
                .unwrap();
        }

        let start = time_duration();
        for _ in 0..size {
            table
                .alloc_fd_than(FdInfo::new(testfile.clone(), OpenFlags::empty()), 11)
                .unwrap();
        }

        (start, time_duration())
    };

    // 运行测试
    const TEST_SIZE: usize = 10_00;
    println!("\n[Test 1] Sequential allocation ({} FD)", TEST_SIZE);
    let (start, end) = test_sequential_allocation(TEST_SIZE);
    println!(
        "Time: start: {:?}, end: {:?}, usetime = {:?}",
        start,
        end,
        end - start
    );

    println!("\n[Test 2] Random reuse ({} FD, 50% reuse)", TEST_SIZE);
    let (start, end) = test_random_reuse(TEST_SIZE);
    println!(
        "Time: start: {:?}, end: {:?}, usetime = {:?}",
        start,
        end,
        end - start
    );

    const TURNOVER_CYCLES: usize = 1_000;
    const BATCH_SIZE: usize = 1000;
    println!(
        "\n[Test 3] High turnover ({} cycles × {} FD)",
        TURNOVER_CYCLES, BATCH_SIZE
    );
    let (start, end) = test_high_turnover(TURNOVER_CYCLES, BATCH_SIZE);
    println!(
        "Time: start: {:?}, end: {:?}, usetime = {:?}",
        start,
        end,
        end - start
    );

    println!("\n[Test 4] Allocate FD > 10 ({} FD)", TEST_SIZE);
    let (start, end) = test_alloc_than(TEST_SIZE);
    println!(
        "Time: start: {:?}, end: {:?}, usetime = {:?}",
        start,
        end,
        end - start
    );

    // 内存使用分析
    println!("\nMemory usage analysis:");
    let table = FdTable::new();
    let base_size = core::mem::size_of::<FdTable>();
    println!("- Empty FD table: {} bytes", base_size);

    let mut large_table = FdTable::new();
    large_table.rlimit.rlim_cur = 10_000;
    large_table.table.resize(10_000, FdInfo::new_bare());
    let full_size = core::mem::size_of_val(&large_table);
    println!("- 10,000 FD table: {} bytes", full_size);

    println!("\nPerformance tests completed!");
}
