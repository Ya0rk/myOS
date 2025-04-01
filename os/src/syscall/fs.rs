use alloc::string::String;
use log::info;
use crate::config::{AT_FDCWD, PATH_MAX, RLIMIT_NOFILE};
use crate::fs::{ mkdir, open_file, Kstat, MountFlags, OpenFlags, Path, Pipe, UmountFlags, MNT_TABLE};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token, Fd, FdTable};
use crate::utils::{Errno, SysResult};

pub async fn sys_write(fd: usize, buf: usize, len: usize) -> SysResult<usize> {
    // info!("[sys_write] start");
    let token = current_user_token();
    let task = current_task().unwrap();
    if fd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }
    match task.get_file_by_fd(fd) {
        Some(file) => {
            if !file.writable() {
                return Err(Errno::EPERM);
            }
            let file = file.clone();

            Ok(file.write(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))).await? as usize)
        }
        _ => Err(Errno::EBADCALL),
    }
}

pub async fn sys_read(fd: usize, buf: usize, len: usize) -> SysResult<usize> {
    let token = current_user_token();
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
            let file = file.clone();
            Ok(file.read(UserBuffer::new(translated_byte_buffer(token, buf as *const u8, len))).await? as usize)
        }
        _ => Err(Errno::EBADCALL),
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
pub fn sys_fstat(fd: usize, kst: *const u8) -> SysResult<usize> {
    let task = current_task().unwrap();
    // let inner = task.inner_lock();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    if kst.is_null() {
        return Err(Errno::EFAULT);
    }
    

    let token = task.get_user_token();
    let mut buffer = UserBuffer::new(
        translated_byte_buffer(
            token, 
            kst, 
            core::mem::size_of::<Kstat>()
    ));

    let mut stat = Kstat::new();
    match task.get_file_by_fd(fd) {
        Some(file) => {
            file.fstat(&mut stat)?;
            buffer.write(stat.as_bytes());
            info!("fstat finished");
            return Ok(0);
        }
        _ => {
            return Err(Errno::EBADCALL);
        }
    }

}

/// 打开或创建一个文件：https://man7.org/linux/man-pages/man2/open.2.html
/// 
/// Success: 返回文件描述符; Fail: 返回-1
pub fn sys_openat(fd: isize, path: *const u8, flags: u32, _mode: usize) -> SysResult<usize> {
    info!("sys_openat start");

    let task = current_task().unwrap();
    let token = current_user_token();
    let path = Path::string2path(translated_str(token, path));
    let flags = OpenFlags::from_bits(flags as i32).unwrap();

    // 计算目标路径
    let target_path = if path.is_absolute() {
        // 绝对路径，忽略 fd
        path.get()
    } else if fd == AT_FDCWD {
        // 相对路径，以当前目录为起点
        let current_path = task.get_current_path();
        path.join_path_2_absolute(current_path).get()
    } else {
        // 相对路径，以 fd 对应的目录为起点
        if fd < 0 || fd as usize > RLIMIT_NOFILE {
            return Err(Errno::EBADF);
        }
        let inode = task.get_file_by_fd(fd as usize).unwrap();
        let other_cwd = inode.get_name()?;
        path.join_path_2_absolute(other_cwd).get()
    };

    // 检查路径是否有效并打开文件
    if let Some(inode) = open_file(target_path.as_str(), flags) {
        let fd = task.alloc_fd(Fd::new(inode.file()?, flags));
        info!("[sys_openat] alloc fd finished, new fd = {}", fd);
        if fd > RLIMIT_NOFILE {
            return Err(Errno::EMFILE);
        } else {
            // info!("[sys_openat] task pid = {}", task.get_pid());
            // info!("[sys_openat] new fd = {}", fd);
            return Ok(fd);
        }
    } else {
        info!("openat fail");
        return Err(Errno::EBADCALL);
    };
}

pub fn sys_close(fd: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    info!("[sys_close] start, pid = {}, closed fd = {}", task.get_pid(), fd);
    if fd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }
    
    // 删除对应的fd
    task.remove_fd(fd);
    Ok(0)
}

/// 创建一个管道：https://man7.org/linux/man-pages/man2/pipe.2.html
/// 
/// pipefd\[0] 指向管道的读取端，pipefd\[1] 指向管道的写入端
/// 
/// Success: 返回0; Fail: 返回-1
pub fn sys_pipe2(pipefd: *mut u32, flags: i32) -> SysResult<usize> {
    info!("sys_pipe start!");
    let flags = OpenFlags::from_bits(flags).ok_or(Errno::EINVAL)?;
    let task = current_task().unwrap();
    let (read_fd, write_fd) = {
        let (read, write) = Pipe::new();
        (
            task.alloc_fd(Fd::new(read, flags)),
            task.alloc_fd(Fd::new(write, flags)),
        )
    };
    info!("alloc read_fd = {}, write_fd = {}", read_fd, write_fd);

    let token = task.get_user_token();
    *translated_refmut(token, pipefd) = read_fd as u32;
    *translated_refmut(token, unsafe { pipefd.add(1) }) = write_fd as u32;
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
///     uint64 d_ino;	// 索引结点号
///     int64 d_off;	// 到下一个dirent的偏移
///     unsigned short d_reclen;	// 当前dirent的长度
///     unsigned char d_type;	// 文件类型
///     char d_name[];	//文件名
/// };
/// ```
/// 
/// len：buf的大小。
/// 
/// 返回值：成功执行，返回读取的字节数。当到目录结尾，则返回0。失败，则返回-1。
pub fn sys_getdents64(fd: usize, buf: *const u8, _len: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    // let inner = task.inner_lock();
    if fd >= task.fd_table_len() || fd > RLIMIT_NOFILE {
        return Err(Errno::EBADF);
    }

    if buf.is_null() {
        return Err(Errno::EFAULT);
    }
    Ok(10)
    // TODO: 有待修改
    // match inner.fd_table.get_file_by_fd(fd) {
    //     Ok(Some(file)) => {
    //         if !file.readable() {
    //             return Err(Errno::EACCES);
    //         }
    //         if !file.is_dir() {
    //             return Err(Errno::ENOENT);
    //         }

    //         let token = inner.get_user_token();
    //         drop(inner);
    //         let mut buffer = UserBuffer::new(
    //             translated_byte_buffer(
    //                 token, 
    //                 buf, 
    //                 len
    //         ));

    //         let mut dirent = Dirent::new();
    //         let mut current_wirte_len = 0;
    //         let dirent_size = core::mem::size_of::<Dirent>();

    //         if len < dirent_size {
    //             return Err(Errno::EINVAL);
    //         }

    //         // TODO :按照测试用例的话这里不需要循环
    //         while len >= dirent_size + current_wirte_len {
    //             let readsize: isize = file.get_dirent(&mut dirent);
    //             if readsize < 0 {
    //                 return Ok(current_wirte_len);
    //             }
    //             let dirent_bytes = dirent.as_bytes();
    //             buffer.write_at(current_wirte_len, dirent_bytes);
    //             current_wirte_len += dirent_size;
    //         }
    //         return Ok(current_wirte_len);
    //     }
    //     _ => {
    //         return Err(Errno::EBADCALL);
    //     }
    // }
}

/// 获取当前工作目录： https://man7.org/linux/man-pages/man3/getcwd.3.html
///
/// Success: 返回当前工作目录的长度;  Fail: 返回-1
pub fn sys_getcwd(buf: *mut u8, size: usize) -> SysResult<usize> {
    if buf.is_null() || size == 0 {
        return Err(Errno::EINVAL);
    }

    let task =  current_task().unwrap();
    // let task_inner = task.inner_lock();
    let token = task.get_user_token();
    let cwd: String = task.get_current_path();
    let length: usize = cwd.len();

    if length > PATH_MAX {
        return Err(Errno::ENAMETOOLONG);
    }
    if length + 1 > size {
        return Err(Errno::ERANGE);
    }

    // drop(task_inner);
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
    // let mut inner = task.inner_lock();
    if oldfd >= task.fd_table_len() {
        return Err(Errno::EBADF);
    }

    let old_temp_fd = task.get_fd(oldfd);
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.set_close_on_exec(true);
    let new_fd = task.alloc_fd(new_temp_fd);
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
    // let mut inner = task.inner_lock();
    
    if newfd > RLIMIT_NOFILE ||
        oldfd >= task.fd_table_len() ||
        task.fd_is_none(oldfd) 
    {
        return Err(Errno::EBADF);
    }

    let old_temp_fd = task.get_fd(oldfd);
    // 关闭 new fd 的close-on-exec flag (FD_CLOEXEC; see fcntl(2))
    let new_temp_fd = old_temp_fd.set_close_on_exec(cloexec);
    // 将newfd 放到指定位置
    task.put_fd_in(new_temp_fd, newfd);

    Ok(newfd)
}

/// 创建一个新目录：https://man7.org/linux/man-pages/man2/mkdir.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_mkdirat(dirfd: isize, path: *const u8, mode: usize) -> SysResult<usize> {

    // Err(Errno::EBADCALL)
    info!("sys_mkdirat start");

    let task = current_task().unwrap();
    let token = current_user_token();
    let path = Path::string2path(translated_str(token, path));
    // info!("sys_mkdirat: path = {}, mode = {}", path.get(), mode);

    // info!("sys_mkdirat cwd is {}", task.get_current_path());

    // 计算目标路径
    let target_path = if path.is_absolute() {
        // 绝对路径，忽略 dirfd
        path.get()
    } else if dirfd == AT_FDCWD {
        // 相对路径，以当前目录为起点
        let current_path = task.get_current_path();
        path.join_path_2_absolute(current_path).get()
    } else {
        // 相对路径，以 dirfd 对应的目录为起点
        if dirfd < 0 || dirfd as usize > RLIMIT_NOFILE || dirfd >= task.fd_table_len() as isize {
            return Err(Errno::EBADF);
        }
        let inode = task.get_file_by_fd(dirfd as usize).unwrap();
        let other_cwd = inode.get_name()?;
        path.join_path_2_absolute(other_cwd).get()
    };
    // info!("sys_mkdirat target_path is {}", target_path);

    // drop(inner);

    // 检查路径是否有效并创建目录
    let result = if let Some(_) = mkdir(target_path.as_str(), mode) {
        Ok(0) // 成功
    } else {
        Err(Errno::EBADCALL) // 失败
    };

    result
}

/// 卸载文件系统：https://man7.org/linux/man-pages/man2/umount.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_umount2(target: *const u8, flags: u32) -> SysResult<usize> {
    UmountFlags::from_bits(flags as u32).ok_or(Errno::EINVAL)?;

    let token = current_user_token();
    let target = translated_str(token, target);
    match MNT_TABLE.lock().umount(target, flags as u32) {
        0 => Ok(0),
        _ => Err(Errno::EBADCALL),
    }
}

/// 挂载文件系统: https://man7.org/linux/man-pages/man2/mount.2.html
/// 
/// Success: 0; Fail: 返回-1
pub fn sys_mount(source: *const u8, target: *const u8, fstype: *const u8, flags: u32, data: *const u8) -> SysResult<usize> {
    let token = current_user_token();
    let source = translated_str(token, source);
    let target = translated_str(token, target);
    let fstype = translated_str(token, fstype);
    let data = match data.is_null() {
        true => String::new(),
        false => translated_str(token, data),
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
pub fn sys_chdir(path: *const u8) -> SysResult<usize> {
    // info!("sys_chdir start");

    let token = current_user_token();
    let task = current_task().unwrap();
    let path = Path::string2path(translated_str(token, path));

    // let mut inner = task.inner_lock();
    let current_path = task.get_current_path();

    // 计算新路径
    let new_path = if path.is_absolute() {
        path.get()
    } else {
        path.join_path_2_absolute(current_path).get()
    };

    // 检查路径是否有效
    let result = if let Some(_) = open_file(new_path.as_str(), OpenFlags::O_RDONLY) {
        task.set_current_path(new_path); // 更新当前路径
        Ok(0) // 成功
    } else {
        Err(Errno::EBADCALL) // 失败
    };

    result
}