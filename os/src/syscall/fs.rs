use core::cell::SyncUnsafeCell;
use core::cmp::{max, min};
use core::error;
use core::ops::Add;
use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::vec;
use log::{debug, info};
use lwext4_rust::file;
use crate::fs::ext4::NormalFile;
use crate::hal::config::{AT_FDCWD, PATH_MAX, RLIMIT_NOFILE};
use crate::fs::{ chdir, mkdir, open, resolve_path, AbsPath, Dentry, Dirent, FileClass, FileTrait, InodeType, Kstat, MountFlags, OpenFlags, Pipe, RenameFlags, Statx, Stdout, StxMask, UmountFlags, MNT_TABLE, SEEK_CUR};
use crate::mm::user_ptr::{user_cstr, user_ref_mut, user_slice, user_slice_mut};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::sync::time::{UTIME_NOW, UTIME_OMIT};
use crate::sync::{time_duration, TimeSpec, TimeStamp, CLOCK_MANAGER};
use crate::syscall::ffi::{IoVec, StatFs};
use crate::task::{current_task, current_user_token, FdInfo, FdTable};
use crate::utils::{backtrace, Errno, SysResult};
use super::ffi::{FaccessatMode, FcntlArgFlags, FcntlFlags, AT_REMOVEDIR};

pub async fn sys_write(fd: usize, buf: usize, len: usize) -> SysResult<usize> {
    // info!("[sys_write] start");
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match task.get_file_by_fd(fd) {
        Some(file) => {
            if !file.writable() {
                return Err(Errno::EPERM);
            }
            // let file = file.clone();
            let buf = unsafe { core::slice::from_raw_parts(buf as *mut u8, len) };
            Ok(file.write(buf).await? as usize)
        }
        _ => Err(Errno::EBADF),
    }
}

pub async fn sys_read(fd: usize, buf: usize, len: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() {
        // info!("[sys_read] task pid = {}", task.get_pid());
        // info!("[sys_read] fd = {}, but fd len is {}", fd, task.fd_table_len());
        return Err(Errno::EBADF);
    }
    match task.get_file_by_fd(fd) {
        Some(file) => {
            if !file.readable() {
                return Err(Errno::EPERM);
            }
            // let file = file.clone();
            let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
            Ok(file.read(buf).await? as usize)
        }
        _ => Err(Errno::EBADF),
    }
}

/// system call reads iovcnt buffers from the file associated 
/// with the file descriptor fd into the buffers described by iov
/// iov: 指向一个结构体数组，结构体的定义如下：
/// ```
/// struct iovec {
///    void *iov_base;	// 指向数据缓冲区的指针
///   size_t iov_len;	// 缓冲区的长度
/// };
///```
/// len: 数组的长度
pub async fn sys_readv(fd: usize, iov: usize, iovcnt: usize) -> SysResult<usize> {
    // info!("[sys_readv] start");
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let mut res = 0;
    if fd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match task.get_file_by_fd(fd) {
        Some(file) => {
            if !file.readable() {
                return Err(Errno::EPERM);
            }
            // 将iov中的结构体一个个取出，转化为UserBuffer
            for i in 0..iovcnt {
                let iov_st = iov.add(core::mem::size_of::<IoVec>() * i) as *mut IoVec;
                let len = (unsafe { *iov_st }).iov_len;
                if len == 0 {
                    continue;
                }
                let base = (unsafe { *iov_st }).iov_base;
                let buffer = unsafe {core::slice::from_raw_parts_mut(base as *mut u8, len)};
                let read_len = file.read(buffer).await?;
                res += read_len;
            }
            Ok(res)
        }
        _ => Err(Errno::EBADF),
    }
}

/// 和sys_readv相反，将数据从iov中写入到文件中
/// system call writes iovcnt buffers from the file associated
/// with the file descriptor fd into the buffers described by iov
pub async fn sys_writev(fd: usize, iov: usize, iovcnt: usize) -> SysResult<usize> {
    // info!("[sys_writev] fd = {}", fd);
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let mut res = 0;
    if fd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match task.get_file_by_fd(fd) {
        Some(file) => {
            if !file.writable() {
                info!("no writeable");
                return Err(Errno::EPERM);
            }
            // 将iov中的结构体一个个取出，转化为UserBuffer
            for i in 0..iovcnt {
                let iov_st = iov.add(core::mem::size_of::<IoVec>() * i) as *const IoVec;
                let len = (unsafe { &*iov_st }).iov_len;
                if len == 0 {
                    continue;
                }
                let base = (unsafe { &*iov_st }).iov_base;
                let buffer = unsafe{core::slice::from_raw_parts(base as *const u8, len)};
                // info!("aaaaaaaaaaaa");
                let write_len = file.write(buffer).await?;
                res += write_len;
            }
            // info!("nnnnnnnnnnnn");
            Ok(res)
        }
        _ => Err(Errno::EBADF),
    }
}

/// dirfd：目录文件描述符，指定相对路径的基准目录
/// 
/// 可以是打开的目录文件描述符
/// 
/// 特殊值 AT_FDCWD 表示当前工作目录
/// ```
/// pathname：目标文件的路径名,可以是绝对路径或相对于 dirfd 的相对路径
/// statbuf：用于存储文件状态信息的结构体指针
/// flags：控制函数行为的标志位
/// - AT_SYMLINK_NOFOLLOW：不跟随符号链接（类似 lstat）
/// - AT_EMPTY_PATH：当 pathname 为空字符串时，操作 dirfd 本身
/// ```
pub fn sys_fstatat(
    dirfd: isize,
    pathname: usize,
    statbuf: usize,
    flags: u32
) -> SysResult<usize> {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = user_cstr(pathname.into())?.unwrap();
    debug!("[sys_fsstatat] pathname {},", path);
    let cwd = task.get_current_path();
    info!("[sys_fstatat] start cwd: {}, pathname: {}, flags: {}, dirfd = {}", cwd, path, flags, dirfd);

    // 计算目标路径
    let target_path = if dirfd == AT_FDCWD {
        resolve_path(cwd, path)
    } else {
        // 相对路径，以 dirfd 对应的目录为起点
        if dirfd < 0 || dirfd as usize > RLIMIT_NOFILE || dirfd >= task.fd_table_len() as isize {
            return Err(Errno::EBADF);
        }
        let inode = match task.get_file_by_fd(dirfd as usize) {
            Some(i) => i,
            _ => return Ok(0),
        };
        // let other_cwd = cwd.clone();
        let other_cwd = inode.get_name()?;
        if other_cwd.contains("is pipe file") || other_cwd == String::from("Stdout") {
            return Ok(0);
        }
        resolve_path(other_cwd, path)
    };

    let ptr = statbuf as *mut Kstat;

    let mut tempstat: Kstat = Kstat::new();
    // 检查路径是否有效并打开文件
    match open(target_path, OpenFlags::O_RDONLY) {
        Ok(FileClass::File(file)) => {
            file.fstat(&mut tempstat)?;
            unsafe{ core::ptr::write(ptr, tempstat); }
            return Ok(0);
        }
        Ok(FileClass::Abs(file)) => {
            file.fstat(&mut tempstat)?;
            unsafe{ core::ptr::write(ptr, tempstat); }
            return Ok(0);
        }
        _ => return Err(Errno::ENOENT)
    }
}

/// 功能：获取文件状态；用来将参数fd 所指向的文件状态复制到参数kst 所指向的结构中
/// 
/// 输入：
/// 
/// fd: 文件句柄；
/// kst: 接收保存文件状态的指针；
/// 
/// 返回值：成功返回0，失败返回-1；
pub fn sys_fstat(fd: usize, kst: usize) -> SysResult<usize> {
    info!("[sys_fstat] start");
    let task = current_task().unwrap();
    // let inner = task.inner_lock();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    let ptr = kst as *mut Kstat;
    if ptr.is_null() {
        return Err(Errno::EFAULT);
    }

    let mut stat = Kstat::new();
    match task.get_file_by_fd(fd) {
        Some(file) => {
            file.fstat(&mut stat)?;
            info!("fstat finished fd: {}, stat: {:?}", fd, stat);
            unsafe{ core::ptr::write(ptr, stat); }
            return Ok(0);
        }
        _ => {
            return Err(Errno::EBADF);
        }
    }

}


/// 291号系统调用
/// 
/// 这个函数返回一个文件的信息, 将其存储到statxbuf中, statxbuf是一个指向statx结构体的指针
/// 
/// mask是要求的掩盖码,在内核中的定义是StxMask
/// 
/// 要访问文件的元数据不需要对文件本事有任何权限, 但是如果通过pathname参数指定路径, 那么需要对路径中的每级父目录都有搜索权限
/// 
/// 如果pathname是绝对路径就直接访问
/// 
/// 如果pathname是相对路径, 且dirfd是AT_FDCWD, 那么就从当前工作目录开始访问
/// 
/// 如果pathname是相对路径, 且dirfd不是AT_FDCWD, 那么就从dirfd指定的目录开始访问
/// 
/// 暂时忽略_mask就全部塞进去算了
pub fn sys_statx(dirfd: i32, pathname: usize, flags: u32, mask: u32, statxbuf: usize) -> SysResult<usize> {
    info!("[sys_statx] start");
    let dirfd = dirfd as isize;
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = user_cstr(pathname.into())?.unwrap();
    let cwd = task.get_current_path();

    // 无效的掩码
    let _mask = StxMask::from_bits(mask).ok_or(Errno::EINVAL)?;
    if _mask.contains(StxMask::STATX__RESERVED) {
        return Err(Errno::EINVAL);
    }

    info!("[sys_statx] start cwd: {}, pathname: {}, flags: {}", cwd, path, flags);

    // 计算目标路径
    let target_path = if dirfd == AT_FDCWD {
        resolve_path(cwd, path)
    } else {
        // 相对路径，以 dirfd 对应的目录为起点
        if dirfd < 0 || dirfd as usize > RLIMIT_NOFILE || dirfd >= task.fd_table_len() as isize {
            return Err(Errno::EBADF);
        }
        let inode = task.get_file_by_fd(dirfd as usize).expect("[sys_statx] not found fd");
        let other_cwd = inode.get_name()?;
        resolve_path(other_cwd, path)
    };

    let mut stat = Kstat::new();
    // 检查路径是否有效并打开文件
    match open(target_path, OpenFlags::O_RDONLY) {
        Ok(FileClass::File(file)) => {
            file.fstat(&mut stat)?;
            let mut statx: Statx = stat.into();
            statx.set_mask(mask);
            unsafe{ core::ptr::write(statxbuf as *mut Statx, statx); } // 这里没有做长度检查
            debug_point!("");
            return Ok(0);
        }
        Ok(FileClass::Abs(file)) => {
            file.fstat(&mut stat)?;
            let mut statx: Statx = stat.into();
            statx.set_mask(mask);
            unsafe{ core::ptr::write(statxbuf as *mut Statx, statx); }
            debug_point!("");
            return Ok(0);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

/// 打开或创建一个文件：https://man7.org/linux/man-pages/man2/open.2.html
/// 
/// Success: 返回文件描述符; Fail: 返回-1
pub fn sys_openat(fd: isize, path: usize, flags: u32, _mode: usize) -> SysResult<usize> {
    info!("[sys_openat] start");
    if path == 0 {
        info!("[sys_openat] path ptr is null, fault.");
        return Err(Errno::EFAULT); 
    }
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = user_cstr(path.into())?.unwrap();
    let flags = OpenFlags::from_bits(flags as i32).ok_or(Errno::EINVAL)?;
    let cwd = task.get_current_path();
    info!("[sys_openat] path = {}, flags = {:?}", path, flags);

    // 计算目标路径
    let target_path = if fd == AT_FDCWD {
        resolve_path(cwd, path)
    } else {
        // 相对路径，以 fd 对应的目录为起点
        if fd < 0 || fd as usize > RLIMIT_NOFILE {
            return Err(Errno::EBADF);
        }
        let inode = task.get_file_by_fd(fd as usize).unwrap();
        let other_cwd = inode.get_name()?;
        info!("[sys_openat] other cwd = {}", other_cwd);
        resolve_path(other_cwd, path)
    };

    // 检查路径是否有效并打开文件
    match open(target_path, flags) {
        Ok(fileclass) => {
            let fd = match fileclass {
                FileClass::File(file) => {
                    task.alloc_fd(FdInfo::new(file, flags))?
                },
                FileClass::Abs(file) => {
                    task.alloc_fd(FdInfo::new(file, flags))?
                },
                _ => { unreachable!() }
            };
            info!("[sys_openat] taskid = {}, alloc fd finished, new fd = {}",task.get_pid(), fd);
            if fd > RLIMIT_NOFILE {
                return Err(Errno::EMFILE);
            } else {
                return Ok(fd);
            }
        }
        Err(e) => {
            info!("[sys_openat] open file failed: {:?}", e);
            return Err(e);
        }
    }
}

pub fn sys_close(fd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    info!("[sys_close] start, pid = {}, closed fd = {}", task.get_pid(), fd);
    if fd >= task.fd_table_len() {
        return Err(Errno::EMFILE);
    }
    
    // 删除对应的fd
    task.remove_fd(fd)?;
    Ok(0)
}

/// 创建一个管道：https://man7.org/linux/man-pages/man2/pipe.2.html
/// 
/// pipefd\[0] 指向管道的读取端，pipefd\[1] 指向管道的写入端
/// 
/// Success: 返回0; Fail: 返回-1
pub fn sys_pipe2(pipefd: *mut u32, flags: i32) -> SysResult<usize> {
    info!("[sys_pipe] start!");
    let flags = OpenFlags::from_bits(flags).ok_or(Errno::EINVAL)?;
    let task = current_task().unwrap();
    let (read_fd, write_fd) = {
        let (read, write) = Pipe::new();
        (
            task.alloc_fd(FdInfo::new(read.clone(),  OpenFlags::O_RDONLY))?,
            task.alloc_fd(FdInfo::new(write.clone(), OpenFlags::O_WRONLY))?,
        )
    };
    info!("[sys_pipe] taskid = {}, alloc read_fd = {}, write_fd = {}", task.get_pid(), read_fd, write_fd);

    let token = task.get_user_token();
    unsafe {
        core::ptr::write(pipefd, read_fd as u32);
        core::ptr::write(pipefd.add(1), write_fd as u32);
    }
    Ok(0)
}

/// 功能：获取目录的条目;
///
/// 输入：
///
/// fd：所要读取目录的文件描述符。
/// 
/// buf：一个缓存区，用于保存所读取目录的信息。缓存区的结构如下：
/// 
/// ```
/// struct dirent {
///     uint64 d_ino;	            // 索引结点号
///     int64 d_off;	            // 到下一个dirent的偏移
///     unsigned short d_reclen;	// 当前dirent的长度
///     unsigned char d_type;	    // 文件类型
///     char d_name[];	            // 文件名
/// };
/// ```
/// 
/// len：buf的大小。
/// 
/// 返回值：成功执行，返回读取的字节数。当到目录结尾，则返回0。失败，则返回-1。
pub fn sys_getdents64(fd: usize, buf: usize, len: usize) -> SysResult<usize> {
    info!("[sys_getdents64] start fd: {}, len: {}", fd, len);
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        log::error!("[sys_getdents64] fd {} invalid", fd);
        return Err(Errno::EBADF);
    }

    let buf = buf as *mut u8;
    if buf.is_null() {
        log::error!("[sys_getdents64] buf is null");
        return Err(Errno::EFAULT);
    }
    // TODO: 有待修改

    let token = task.get_user_token();
    let file = task.get_file_by_fd(fd).unwrap();
    let res = file.read_dents(buf as usize, len);
    info!("[sys_getdents64] return = {}", res);
    Ok(res)
}

/// 获取当前工作目录： https://man7.org/linux/man-pages/man3/getcwd.3.html
///
/// Success: 返回当前工作目录的长度;  Fail: 返回-1
pub fn sys_getcwd(buf: usize, size: usize) -> SysResult<usize> {
    info!("[sys_getcwd] start");
    let ptr = buf as *mut u8;
    if ptr.is_null() || size == 0 {
        return Err(Errno::EINVAL);
    }

    let task =  current_task().unwrap();
    // let task_inner = task.inner_lock();
    let token = task.get_user_token();
    let cwd = task.get_current_path();
    let length: usize = cwd.len() + 1;
    info!("[sys_getcwd] cwd is {}", cwd);
    let cs_cwd: CString = CString::new(cwd).expect("can translate to cstring");

    if length > PATH_MAX {
        return Err(Errno::ENAMETOOLONG);
    }
    if length > size {
        return Err(Errno::ERANGE);
    }

    // drop(task_inner);
    // TODO: 检测当前cwd是不是被unlinked： ENOENT The current working directory has been unlinked.
    // end
    let write_len = min(length, size);
    let buf = unsafe{ core::slice::from_raw_parts_mut(ptr, write_len) };
    buf.copy_from_slice(cs_cwd.as_bytes_with_nul());

    Ok(length)
}

/// 创建一个现有文件描述符的副本：https://man7.org/linux/man-pages/man2/dup.2.html
/// 
/// Success: 返回新的文件描述符; Fail: 返回-1
pub fn sys_dup(oldfd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    info!("[sys_dup] pid = {}, oldfd = {}", task.get_pid(), oldfd);
    // let mut inner = task.inner_lock();

    let old_temp_fd = task.get_fd(oldfd);
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.off_Ocloexec(true);
    let new_fd = task.alloc_fd(new_temp_fd)?;
    // drop(inner);
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
    info!("[sys_dup3] start");
    if oldfd == newfd {
        return Err(Errno::EINVAL);
    }

    // 判断flags是否合法
    let flag = OpenFlags::from_bits(flags as i32).unwrap();
    let cloexec = {
        match flag {
            OpenFlags::O_CLOEXEC => Some(true),
            _ => Some(false),
        }
    }.ok_or(Errno::EINVAL)?;

    let task = current_task().unwrap();
    info!("[sys_dup3] start, oldfd={oldfd}, newfd={newfd}, taskid = {}", task.get_pid());
    
    if newfd > RLIMIT_NOFILE
    {
        return Err(Errno::EBADF);
    }

    let old_temp_fd = task.get_fd(oldfd);
    if old_temp_fd.is_none() { return Err(Errno::EBADF); }
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.clone().off_Ocloexec(!cloexec);
    // info!("[sys_dup3] old file name = {}, oldfd = {}", old_temp_fd.clone().file.unwrap().get_name()?, oldfd);
    // 将newfd 放到指定位置
    task.put_fd_in(new_temp_fd, newfd);

    Ok(newfd)
}

/// 创建一个新目录：https://man7.org/linux/man-pages/man2/mkdir.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_mkdirat(dirfd: isize, path: usize, mode: usize) -> SysResult<usize> {
    info!("[sys_mkdirat] start");

    let task = current_task().unwrap();
    let token = current_user_token();
    let path = user_cstr(path.into())?.unwrap();
    let cwd = task.get_current_path();

    // 计算目标路径
    let target_path = if dirfd == AT_FDCWD {
        // 相对路径，以当前目录为起点
        resolve_path(cwd, path)
    } else {
        // 相对路径，以 dirfd 对应的目录为起点
        if dirfd < 0 || dirfd as usize > RLIMIT_NOFILE || dirfd >= task.fd_table_len() as isize {
            return Err(Errno::EBADF);
        }
        let inode = task.get_file_by_fd(dirfd as usize).unwrap();
        let other_cwd = inode.get_name()?;
        resolve_path(other_cwd, path)
    };
    // info!("sys_mkdirat target_path is {}", target_path);

    // TODO
    // 返回错误码有问题,应当返回EEXIST:目录存在;EACCES:权限不足;EROFS:文件系统只读;
    // ENOSPC:没有足够的空间;ENAMETOOLONG:路径过长;ENOTDIR:不是目录;
    // ELOOP:符号链接过多;ENOSPC:没有足够的空间;EFAULT:路径错误;等

    // 检查路径是否有效并创建目录
    match mkdir(target_path, mode) {
        Ok(_) => Ok(0), // 成功
        Err(e) => Err(e)
    }
}

// TODO: 有待完善，利用好flag，修改umount参数为AbsPath
/// 卸载文件系统：https://man7.org/linux/man-pages/man2/umount.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_umount2(target: usize, flags: u32) -> SysResult<usize> {
    info!("[sys_umount2] start");
    let ufg = UmountFlags::from_bits(flags as u32).ok_or(Errno::EINVAL)?;
    if ufg.contains(UmountFlags::MNT_EXPIRE)
        && (ufg.contains(UmountFlags::MNT_DETACH) || ufg.contains(UmountFlags::MNT_FORCE))
    {
        return Err(Errno::EINVAL);
    }

    let token = current_user_token();
    let target = user_cstr(target.into())?.unwrap();
    match MNT_TABLE.lock().umount(target, flags as u32) {
        0 => Ok(0),
        _ => Err(Errno::EBADCALL),
    }
}

/// 挂载文件系统: https://man7.org/linux/man-pages/man2/mount.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_mount(source: usize, target: usize, fstype: usize, flags: u32, data: usize) -> SysResult<usize> {
    info!("[sys_mount] start");
    let token = current_user_token();
    let source = user_cstr(source.into())?.unwrap();
    let target  = user_cstr(target.into())?.unwrap();
    let fstype = user_cstr(fstype.into())?.unwrap();
    let data = match (data as *const u8).is_null() {
        true => String::new(),
        false => user_cstr(data.into())?.unwrap(),
    };
    // info!("sys_mount: source = {}, target = {}, fstype = {}, flags = {}, data = {}", source, target, fstype, flags, data);

    let check_flags = MountFlags::from_bits(flags).unwrap();

    let mut mnt_table = MNT_TABLE.lock();

    if check_flags.contains(MountFlags::MS_REMOUNT) && !mnt_table.is_mounted(source.clone())
        || check_flags.contains(MountFlags::MS_MOVE) && source == "/"
    {
        return Err(Errno::EINVAL);
    }

    match mnt_table.mount(source, target, fstype, flags as u32, data) {
        0 => Ok(0),
        _ => Err(Errno::EBADCALL),
    }
}

/// 切换到指定目录: https://man7.org/linux/man-pages/man2/chdir.2.html
/// 
/// 输入： path:  需要切换到的路径
/// 
/// Success: 返回0； 失败： 返回-1；
pub fn sys_chdir(path: usize) -> SysResult<usize> {
    info!("[sys_chdir] start");

    let token = current_user_token();
    let task = current_task().unwrap();
    let path = user_cstr(path.into())?.unwrap();
    info!("[sys_chidr] path = {}", path);

    // let mut inner = task.inner_lock();
    let current_path = task.get_current_path();

    // 计算新路径
    let target_path = resolve_path(current_path, path);

    // 检查路径是否有效
    chdir(target_path.clone())?;
    task.set_current_path(target_path.get()); // 更新当前路径
    Ok(0) // 成功
}


pub fn sys_unlinkat(fd: isize, path: usize, flags: u32) -> SysResult<usize> {
    // info!("[sys_unlinkat] start");
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let path = user_cstr(path.into())?.unwrap();
    // info!("[sys_unlink] start path = {}", path);
    let base = task.get_current_path();
    info!("[sys_unlinkat] start fd: {}, base: {}, path: {}, flags: {}", fd, base, path, flags);

    let target_path = resolve_path(base, path);

    match open(target_path.clone(), OpenFlags::O_RDWR) {
        Ok(file_class) => {
            let file = file_class.file()?;
            // info!("[unlink] file path = {}", file.path);
            let is_dir = file.is_dir();
            if is_dir && flags != AT_REMOVEDIR {
                return Err(Errno::EISDIR);
            }
            if flags == AT_REMOVEDIR && !is_dir {
                return Err(Errno::ENOTDIR);
            }
            file.get_inode().unlink(&target_path.get());
        }
        Err(e) => {
            // info!("[sys_unlinkat] open file failed: {:?}", e);
            // if e == Errno::ENOENT {
            //     // 如果文件不存在，且flags为AT_REMOVEDIR，则返回成功
            //     if flags == AT_REMOVEDIR {
            //         return Ok(0);
            //     }
            //     return Err(Errno::ENOENT);
            // } else if e == Errno::EISDIR && flags != AT_REMOVEDIR {
            //     return Err(Errno::EISDIR);
            // } else if e == Errno::ENOTDIR && flags == AT_REMOVEDIR {
            //     return Err(Errno::ENOTDIR);
            // } else {
            //     return Err(e);
            // }
            return Err(e);
        }
    }
    info!("[sys_unlink] finished");
    
    Ok(0)
}

/// TODO:这里的rename好像没有真正实现
pub fn sys_renameat2(olddirfd: isize, oldpath: usize, newdirfd: isize, newpath: usize, flags: u32) -> SysResult<usize> {
    let task = current_task().unwrap();
    let flags = RenameFlags::from_bits(flags).ok_or(Errno::EINVAL)?;
    let old_path = user_cstr(oldpath.into())?.unwrap();
    let new_path = user_cstr(newpath.into())?.unwrap();
    let cwd = task.get_current_path();
    info!("[sys_renameat2] start olddirfd: {}, old: {}, newdirfd: {}, new: {} ", &olddirfd, &old_path, &newdirfd, &new_path);

    let old_path = if olddirfd == AT_FDCWD {
        resolve_path(cwd.clone(), old_path)
    } else {
        match task.get_file_by_fd(olddirfd as usize) {
            Some(file) => {
                resolve_path(file.get_name()?, old_path)
            }
            None => {
                // debug_point!("[sys_renameat2] return EBADF");
                return Err(Errno::EBADF);
            }
        }
    };

    let new_path = if newdirfd == AT_FDCWD {
        resolve_path(cwd.clone(), new_path)
    } else {
        match task.get_file_by_fd(newdirfd as usize) {
            Some(file) => {
                resolve_path(file.get_name()?, new_path)
            }
            None => {
                // debug_point!("[sys_renameat2] return EBADF");
                return Err(Errno::EBADF);
            }
        }
    };
    // 简单的实现, 当目标路径存在文件的时候就返回存在
    if let Ok(file) = open(new_path.clone(), OpenFlags::O_RDWR) {
        // debug_point!("[sys_renameat2] return EEXIST");
        return Err(Errno::EEXIST);
    }

    if let Ok(file) = open(old_path.clone(), OpenFlags::O_RDWR) {
        let old_inode = file.file()?.get_inode();
        if let Ok(_) = old_inode.rename(&old_path.get(), &new_path.get()) {
            // 如果重命名成功，返回0
            // debug_point!("[sys_renameat2] return Ok(0)");
            return Ok(0);
        } else {
            // 如果重命名失败，返回错误
            // debug_point!("[sys_renameat2] return EACCES");
            return Err(Errno::EACCES);
        }
    } else {
        // debug_point!("[sys_renameat2] return ENOENT");
        return Err(Errno::ENOENT);
    }
    // debug_point!("[sys_renameat2] return Ok(0)");
    Ok(0)
}

/// make a new name for a file: a hard link
pub fn sys_linkat(olddirfd: isize, oldpath: usize, newdirfd: isize, newpath: usize, flags: u32) -> SysResult<usize> {
    // info!("[sys_linkat] start");
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let old_path = user_cstr(oldpath.into())?.unwrap();
    let new_path = user_cstr(newpath.into())?.unwrap();
    let cwd = task.get_current_path();
    // info!("[sys_linkat] start olddirfd: {}, oldpath: {}, newdirfd: {}, newpath: {}", &olddirfd, &old_path, &newdirfd, &new_path);

    let old_path = if olddirfd == AT_FDCWD {
        resolve_path(cwd.clone(), old_path)
    } else {
        match task.get_file_by_fd(olddirfd as usize) {
            Some(file) => {
                resolve_path(file.get_name()?, old_path)
            }
            None => {
                return Err(Errno::EBADF);
            }
        }
    };

    if let Ok(inode) = Dentry::get_inode_from_path(&old_path.get()) {
        if inode.node_type() == InodeType::Dir {
            return Err(Errno::EISDIR);
        }
    }

    let new_path = if newdirfd == AT_FDCWD {
        resolve_path(cwd.clone(), new_path)
    } else {
        match task.get_file_by_fd(newdirfd as usize) {
            Some(file) => {
                resolve_path(file.get_name()?, new_path)
            }
            None => {
                return Err(Errno::EACCES);
            }
        }
    };

    if olddirfd == AT_FDCWD {
        if let Ok(file_class) = open(old_path, OpenFlags::O_RDWR) {
            let file = file_class.file()?;
            let has_same = file.is_child(&new_path.get());
            if has_same {
                return Err(Errno::EEXIST);
            }
            file.get_inode().link(&new_path.get());
        }
    }
    Ok(0)
}

/// copies data between one file descriptor and another
pub async fn sys_sendfile(out_fd: usize, in_fd: usize, offset: usize, count: usize) -> SysResult<usize> {
    info!("[sys_sendfile] start");
    let task = current_task().unwrap();
    let src = task.get_file_by_fd(in_fd).ok_or(Errno::EBADF)?;
    let dest = task.get_file_by_fd(out_fd).ok_or(Errno::EBADF)?;
    if !src.readable() || !dest.writable() {
        return Err(Errno::EPERM);
    }
    
    let mut len: usize = 0;
    let mut buf = vec![0u8; count];
    let mut new_offset = offset;
    if new_offset != 0 { panic!("not implement") };

    loop {
        let read_size = src.read(&mut buf).await?;
        if read_size == 0 {
            break;
        }
        let write_size = dest.write(&buf[0..read_size]).await?;
        if read_size != write_size {
            return Err(Errno::EIO);
        }
        new_offset += read_size;
        len += read_size;
    }

    // If offset is not NULL, then sendfile() does not modify the file offset of in_fd; 
    // otherwise the file offset is adjusted to reflect the number of bytes read from in_fd.
    if offset != 0 {
        // 重新设置offset：
        let token = task.get_user_token();
        src.lseek(len as isize, SEEK_CUR).unwrap();
        let ptr = user_ref_mut(offset.into())?.unwrap();
        *ptr = new_offset;
        // *translated_refmut(token, offset as *mut usize) = new_offset;
    }
    info!("[sys_sendfile] finished");
    Ok(len)
}

/// determine accessibility of a file relative to directory file descriptor
/// If pathname is a symbolic link, it is dereferenced.
pub fn sys_faccessat(
    dirfd: isize,
    pathname: usize,
    mode: u32,
    _flags: u32,
) -> SysResult<usize> {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let mut path = user_cstr(pathname.into())?.unwrap();
    info!("[sys_faccessat] start dirfd: {}, pathname: {}", dirfd, path);
    let mode = FaccessatMode::from_bits(mode).ok_or(Errno::EINVAL)?;
    let cwd = task.get_current_path();
    
    let abs = if dirfd == AT_FDCWD {
        // 相对路径，以当前目录为起点
        resolve_path(cwd, path)
    } else {
        // 相对路径，以 fd 对应的目录为起点
        if dirfd < 0 || dirfd as usize > RLIMIT_NOFILE {
            return Err(Errno::EBADF);
        }
        let inode = current_task().unwrap().get_file_by_fd(dirfd as usize).expect("[sys_faccessat] get file by fd failed");
        let other_cwd = inode.get_name()?;
        resolve_path(other_cwd, path)
    };

    
    if let Ok(file_class) = open(abs, OpenFlags::O_RDONLY) {
        let file = file_class.file()?;
        let inode = file.get_inode();
        if mode.contains(FaccessatMode::F_OK) {
            return Ok(0);
        }
        if mode.contains(FaccessatMode::R_OK) && !file.readable() {
            return Err(Errno::EACCES);
        }
        if mode.contains(FaccessatMode::W_OK) && !file.writable() {
            return Err(Errno::EACCES);
        }
        if mode.contains(FaccessatMode::X_OK) && !file.executable() {
            return Err(Errno::EACCES);
        }
    } else {
        return Err(Errno::ENOENT);
    }
    Ok(0)
}

/// repositions the file offset of the open file description
/// associated with the file descriptor fd to the argument offset
/// according to the directive whence as follows
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> SysResult<usize> {
    info!("[sys_lseek] start");
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }
    let file = task.get_file_by_fd(fd).unwrap();
    file.lseek(offset, whence)
}

/// TODO(YJJ): 有待完善
/// 用于修改某个文件描述符的属性
/// 第1个参数fd为待修改属性的文件描述符，第2个参数cmd为对应的操作命令，第3个参数为cmd的参数
pub fn sys_fcntl(fd: usize, cmd: u32, arg: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let cmd = FcntlFlags::from_bits(cmd).ok_or(Errno::EINVAL)?;
    info!("[sys_fcntl] start, fd = {}, cmd = {:?}", fd, cmd);
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }
    
    match cmd {
        // F_SETFL：设置文件状态标志。它首先从参数arg中获取标志，然后设置文件描述符的标志。
        FcntlFlags::F_SETFL => {
            if let Some(file) = task.get_file_by_fd(fd) {
                file.set_flags(OpenFlags::from_bits(arg as i32).ok_or(Errno::EINVAL)?);
            }
            return Ok(0);
        }
        // Currently, only one such flag is defined: FD_CLOEXEC (value: 1)
        // todo  Ok(file.available() as isize);
        // F_GETFD和F_GETFL：获取文件描述符的标志。它首先从文件描述符表中获取文件描述符的信息，
        // 然后返回文件描述符的标志。
        FcntlFlags::F_GETFD | FcntlFlags::F_GETFL => {
            // Return (as the function result) the file descriptor flags; arg is ignored.
            if let Some(file) = task.get_file_by_fd(fd) {
                let flags = file.get_flags();
                if flags.contains(OpenFlags::O_CLOEXEC) && cmd.contains(FcntlFlags::F_GETFD) {
                    return Ok(FcntlArgFlags::bits(&FcntlArgFlags::FD_CLOEXEC) as usize);
                } else {
                    return Ok(OpenFlags::bits(&flags) as usize);
                }
            }
            return Err(Errno::EBADF);
        }
        // F_SETFD：设置文件描述符的标志。它首先从参数arg中获取标志，然后设置文件描述符的标志。
        FcntlFlags::F_SETFD => {
            // Set the file descriptor flags to the value specified by arg.
            if let Some(file) = task.get_file_by_fd(fd) {
                let new_flags = FcntlArgFlags::from_bits(arg as u32).ok_or(Errno::EINVAL)?;
            }
            return Ok(0);
        }
        // F_DUPFD：复制文件描述符。它首先从文件描述符表中获取文件，然后分配一个新的文件描述符，
        // 并将文件放入新的文件描述符中
        FcntlFlags::F_DUPFD => {
            if let Some(file) = task.get_file_by_fd(fd) {
                let flags = file.get_flags();
                let newfd = task.alloc_fd_than(FdInfo::new(file, flags), arg as usize);
                return Ok(newfd);
            }
            return Err(Errno::EBADF);
        }
        // F_DUPFD_CLOEXEC：复制文件描述符，并设置新文件描述符的CLOEXEC标志。
        // 这意味着当执行新的程序时，这个文件描述符将被关闭。
        FcntlFlags::F_DUPFD_CLOEXEC => {
            if let Some(file) = task.get_file_by_fd(fd) {
                let flags = file.get_flags();
                let newfd = task.alloc_fd_than(FdInfo::new(file, flags | OpenFlags::O_CLOEXEC), arg as usize);
                return Ok(newfd);
            }
            return Err(Errno::EBADF);
        }
        _ => return Err(Errno::EINVAL),
    }
}

/// 改变文件大小
/// 返回值：0、-1
pub fn sys_ftruncate64(fd: usize, length: usize) -> SysResult<usize> {
    info!("[sys_ftruncate64] start");
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }
    let file = task.get_file_by_fd(fd).unwrap();
    file.get_inode().truncate(length);
    Ok(0)
}

/// 可更改现有文件的访问权限
pub fn sys_fchmodat() -> SysResult<usize> {
    info!("[sys_fchmodat] start");
    return Ok(0);
}

/// 从描述符为fd的文件中，从offset位置开始，读取count个字节存入buf中。
/// 如果读取成功，将返回读取的字节数
pub async fn sys_pread64(
    fd: usize,
    buf: usize,
    count: usize,
    offset: usize,
) -> SysResult<usize> {
    info!("[sys_pread64] start");
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }
    let file = task.get_file_by_fd(fd).unwrap();
    if !file.readable() {
        return Err(Errno::EPERM);
    }
    let buffer = unsafe{ core::slice::from_raw_parts_mut(buf as *mut u8, count) };
    file.pread(buffer, offset, count).await
}

/// 在指定偏移量处向文件描述符写入数据的系统调用
/// pwrite64的行为类似于先执行lseek再执行write，但它是一个原子操作，不会被其他线程的文件操作中断
pub async fn sys_pwrite64(
    fd: usize,
    buf: usize,
    count: usize,
    offset: usize,
) -> SysResult<usize> {
    info!("[sys_pwrite64] start");
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }
    let file = task.get_file_by_fd(fd).unwrap();
    if !file.writable() {
        return Err(Errno::EPERM);
    }
    let buffer = unsafe{ core::slice::from_raw_parts(buf as *const u8, count) };
    file.pwrite(buffer, offset, count).await
}

/// change file timestamps with nanosecond precision
///  With utimensat() the file is specified via the pathname given in pathname
/// times[0] specifies the new "last access time" (atime); times[1] specifies the new "last modification time" (mtime)
pub fn sys_utimensat(dirfd: isize, pathname: usize, times: *const [TimeSpec; 2], flags: i32) -> SysResult<usize> {
    info!("[sys_utimensat] start fd: {}, path: {:#X}", dirfd, pathname);
    let task = current_task().unwrap();

    // 如果pathname不是空，target就是pathname对应文件
    // 如果是空，那么就是dirfd对应文件
    let inode = if pathname != 0 {
        let cwd = task.get_current_path();
        let path = user_cstr(pathname.into())?.unwrap();
        let target_path = resolve_path(cwd, path);
        
        let flags = OpenFlags::from_bits(flags).ok_or(Errno::EINVAL)?;

        open(target_path, OpenFlags::O_RDWR | OpenFlags::O_CREAT)?.file()?.get_inode()
    } else {
        let res = match dirfd {
            AT_FDCWD => { 
                let cwd = task.get_current_path();
                let path = user_cstr(pathname.into())?.unwrap();
                let target_path = resolve_path(cwd, path);

                open(target_path, OpenFlags::O_RDWR | OpenFlags::O_CREAT)?.file()?.get_inode()
            }
            _ => {
                let file = task.get_file_by_fd(dirfd as usize).ok_or(Errno::EBADF)?;
                file.get_inode()
            }
        };
        res
    };
    let mut new_time;
    {new_time = inode.get_timestamp().lock().clone();}
    info!("[sys_utimensat] new_time: \n{:?} \n{:?}",new_time.atime, new_time.mtime);
    if !times.is_null() {
        let user_time = unsafe { &*times };
        // 访问时间
        match user_time[0].tv_nsec {
            UTIME_NOW => {
                // sec设置为当前时间
                new_time.atime = TimeSpec::from(*CLOCK_MANAGER.lock().get(0).unwrap() + time_duration());
                info!("[sys_utimensat] set atime to now {:?}", new_time.atime);
            }
            UTIME_OMIT => {
                // 保持不变
                info!("[sys_utimensat] omit atime {:?}", new_time.atime);
            }
            _ => { 
                new_time.atime = user_time[0]; 
                info!("[sys_utimensat] set atime to {:?}", new_time.atime);
            }
        }
        // 修改时间
        match user_time[1].tv_nsec {
            UTIME_NOW => {
                // sec设置为当前时间
                new_time.mtime = TimeSpec::from(*CLOCK_MANAGER.lock().get(0).unwrap() + time_duration());
                info!("[sys_utimensat] set mtime to now {:?}", new_time.mtime);
            } 
            UTIME_OMIT => { // 保持不变
                // 保持不变
                info!("[sys_utimensat] omit mtime {:?}", new_time.mtime);
            }
            _ => { 
                new_time.mtime = user_time[1]; 
                info!("[sys_utimensat] set mtime to {:?}", new_time.mtime);
            }
        }
    }

    inode.set_timestamps(new_time);

    Ok(0)
}

/// read value of a symbolic link
/// 一个符号链接当中获得真实的路径地址
/// 注意到当前没有真正地实现,返回值全为0,代表不支持该功能
pub fn sys_readlinkat(dirfd: isize, pathname: usize, buf: usize, bufsiz: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let cwd = task.get_current_path();
    let pathname = user_cstr(pathname.into())?.unwrap();
    info!("[sys_readlinkat] start, dirfd: {}, pathname: {}.", dirfd, pathname);

    let target_path = resolve_path(cwd, pathname);
    if !target_path.is_absolute() {
        if dirfd == AT_FDCWD {
            let cwd = task.get_current_path();
            todo!();
            log::error!("case which is no abs path hasn't implement");
            return Ok(0); 
        }
    } else {
        // 由于暂时没有实现软链接,所以先这么做吧,把这个文件重定向到/musl/busybox
        if target_path.get() == "/proc/self/exe" {
            let ub= if let Ok(Some(buf)) = user_slice_mut::<u8>(buf.into(), bufsiz) {
                buf
            } else {
                return Err(Errno::EFAULT);
            };
            let path_bytes = "/musl/busybox\0".as_bytes();
            if path_bytes.len() > bufsiz {
                ub[0..bufsiz].copy_from_slice(&path_bytes[0..bufsiz]);
                return Ok(bufsiz);
            } else {
                ub[0..path_bytes.len()].copy_from_slice(&path_bytes[0..path_bytes.len()]);
                return Ok(path_bytes.len());
            }
        }

        if let Ok(FileClass::File(file)) = open(target_path, OpenFlags::O_RDONLY) {
            let ub= if let Ok(Some(buf)) = user_slice_mut::<u8>(buf.into(), bufsiz) {
                buf
            } else {
                return Err(Errno::EFAULT);
            };
            let c_path = alloc::format!("{}\0", file.path);
            let path_bytes = c_path.as_bytes();
            let len = max(path_bytes.len(), bufsiz);
            ub[0..len].copy_from_slice(&path_bytes[0..len]);
            return Ok(len);
        }
    }
    Ok(0)
}

pub fn sys_statfs(path: usize, buf: usize) -> SysResult<usize> {
    info!("[sys_statfs] start");
    let stat = StatFs::new().to_u8();
    let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, core::mem::size_of::<StatFs>()) };
    buf.copy_from_slice(&stat);

    Ok(0)
}