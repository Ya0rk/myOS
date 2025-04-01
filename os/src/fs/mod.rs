mod devfs;
mod dirent;
mod inode_cache;
mod page_cache;
mod mount;
mod pipe;
mod stat;
mod stdio;
mod vfs;
mod ffi;
mod path;
mod file;
pub mod ext4;

pub use ext4::{root_inode,ls};
pub use ffi::{OpenFlags, UmountFlags, MountFlags};
pub use path::{Path, path_test};
pub use dirent::Dirent;
pub use inode_cache::*;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
pub use stat::Kstat;
pub use vfs::*;
pub use stdio::{Stdin, Stdout};

use devfs::{find_device, open_device_file, register_device};
use ffi::{MOUNTS, MEMINFO, LOCALTIME, ADJTIME};
use file::NormalFile;
use crate::mm::UserBuffer;
use crate::utils::{Errno, SysResult};
use alloc::string::{String, ToString};
use alloc::{sync::Arc, vec::Vec};
use log::{debug, info};

pub const SEEK_SET: usize = 0;
pub const SEEK_CUR: usize = 1;
pub const SEEK_END: usize = 2;

/// 枚举类型，分为普通文件和抽象文件
/// 普通文件File，特点是支持更多类型的操作，包含seek, offset等
/// 抽象文件Abs，抽象文件，只支持File trait的一些操作
#[derive(Clone)]
pub enum FileClass {
    File(Arc<NormalFile>),
    Abs(Arc<dyn FileTrait>),
}

impl FileClass {
    pub fn file(&self) -> Result<Arc<NormalFile>, Errno> {
        match self {
            FileClass::File(f) => Ok(f.clone()),
            FileClass::Abs(_) => Err(Errno::EINVAL),
        }
    }
    pub fn abs(&self) -> Result<Arc<dyn FileTrait>, Errno> {
        match self {
            FileClass::File(_) => Err(Errno::EINVAL),
            FileClass::Abs(f) => Ok(f.clone()),
        }
    }
}

core::arch::global_asm!(include_str!("preload.S"));

// os\src\fs\mod.rs
//将预加载到内存中的程序写入文件根目录
pub async fn flush_preload() {
    extern "C" {
        fn initproc_start();
        fn initproc_end();
    }

    if let Some(FileClass::File(initproc)) = open_file("initproc", OpenFlags::O_CREAT) {
        let mut v = Vec::new();
        v.push(unsafe {
            core::slice::from_raw_parts_mut(
                initproc_start as *mut u8,
                initproc_end as usize - initproc_start as usize,
            ) as &'static mut [u8]
        });
        info!("kkkk");
        initproc.write(UserBuffer::new(v)).await.unwrap();
        info!("ddddd");
    }
}

pub async fn init() {
    flush_preload().await;
    let _ = create_init_files();
}

pub fn list_apps() -> bool{
    println!("/**** APPS ****");
    ls();
    println!("**************/");
    true
}



pub async fn create_init_files() -> SysResult {
    //创建/proc文件夹
    open(
        "/",
        "proc",
        OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
    );
    //创建/proc/mounts文件系统使用情况
    if let Some(FileClass::File(mountsfile)) =
        open("/proc", "mounts", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut mountsinfo = String::from(MOUNTS);
        let mut mountsvec = Vec::new();
        unsafe {
            let mounts = mountsinfo.as_bytes_mut();
            mountsvec.push(core::slice::from_raw_parts_mut(
                mounts.as_mut_ptr(),
                mounts.len(),
            ));
        }
        let mountbuf = UserBuffer::new(mountsvec);
        let mountssize = mountsfile.write(mountbuf).await?;
        debug!("create /proc/mounts with {} sizes", mountssize);
    }
    //创建/proc/meminfo系统内存使用情况
    if let Some(FileClass::File(memfile)) =
        open("/proc", "meminfo", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut meminfo = String::from(MEMINFO);
        let mut memvec = Vec::new();
        unsafe {
            let mem = meminfo.as_bytes_mut();
            memvec.push(core::slice::from_raw_parts_mut(mem.as_mut_ptr(), mem.len()));
        }
        let membuf = UserBuffer::new(memvec);
        let memsize = memfile.write(membuf).await?;
        debug!("create /proc/meminfo with {} sizes", memsize);
    }
    //创建/dev文件夹
    open(
        "/",
        "dev",
        OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
    );
    //注册设备/dev/rtc和/dev/rtc0
    register_device("/dev/rtc");
    register_device("/dev/rtc0");
    //注册设备/dev/tty
    register_device("/dev/tty");
    //注册设备/dev/zero
    register_device("/dev/zero");
    //注册设备/dev/numm
    register_device("/dev/null");
    //创建./dev/misc文件夹
    open(
        "/dev",
        "misc",
        OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
    );
    //注册设备/dev/misc/rtc
    register_device("/dev/misc/rtc");
    //创建/etc文件夹
    open(
        "/",
        "etc",
        OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
    );
    //创建/etc/adjtime记录时间偏差
    if let Some(FileClass::File(adjtimefile)) =
        open("/etc", "adjtime", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut adjtime = String::from(ADJTIME);
        let mut adjtimevec = Vec::new();
        unsafe {
            let adj = adjtime.as_bytes_mut();
            adjtimevec.push(core::slice::from_raw_parts_mut(adj.as_mut_ptr(), adj.len()));
        }
        let adjtimebuf = UserBuffer::new(adjtimevec);
        let adjtimesize = adjtimefile.write(adjtimebuf).await?;
        debug!("create /etc/adjtime with {} sizes", adjtimesize);
    }
    //创建./etc/localtime记录时区
    if let Some(FileClass::File(localtimefile)) =
        open("/etc", "localtime", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut localtime = String::from(LOCALTIME);
        let mut localtimevec = Vec::new();
        unsafe {
            let local = localtime.as_bytes_mut();
            localtimevec.push(core::slice::from_raw_parts_mut(
                local.as_mut_ptr(),
                local.len(),
            ));
        }
        let localtimebuf = UserBuffer::new(localtimevec);
        let localtimesize = localtimefile.write(localtimebuf).await?;
        debug!("create /etc/localtime with {} sizes", localtimesize);
    }
    Ok(())
}

fn create_file(
    abs_path: String,
    parent_path: &str,
    child_name: &str,
    flags: OpenFlags,
) -> Option<FileClass> {
    debug!(
        "[create_file],flags={:?},abs_path={},parent_path={},child_name={}",
        flags, abs_path, parent_path, child_name
    );

    // 一定能找到,因为除了RootInode外都有父结点
    let parent_dir = INODE_CACHE.get(parent_path).unwrap();
    return parent_dir
        .do_create(&abs_path, flags.node_type())
        .map(|vfile| {
            let osinode = NormalFile::new(
                flags,
                Some(Arc::downgrade(&parent_dir)),
                vfile,
                abs_path,
            );
            FileClass::File(Arc::new(osinode))
        });
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<FileClass> {
    open(&"/", path, flags)
}

pub fn open(cwd: &str, path: &str, flags: OpenFlags) -> Option<FileClass> {
    let kpath = Path::string2path(path.to_string());
    
    let new_path = kpath.join_path_2_absolute(cwd.to_string());
    // 目标文件的路径
    let abs_path = new_path.get();

    if find_device(&abs_path) {
        if let Some(device) = open_device_file(&abs_path) {
            return Some(FileClass::Abs(device));
        }
        return None;
    }

    let (parent_path, child_name) = new_path.split_with("/");
    let (parent_path, child_name) = (parent_path.as_str(), child_name.as_str());

    info!(
        "[open] cwd={}, path={}, parent={}, child={}, abs={}",
        cwd, path, parent_path, child_name, &abs_path
    );

    let (parent_inode, _) = if INODE_CACHE.has_inode(parent_path) {
        // info!("aaaa");
        (INODE_CACHE.get(parent_path).unwrap(), child_name)
    } else {
        if cwd == "/" {
            // info!("bbbb");
            (root_inode(), path)
        } else {
            (root_inode().walk(cwd).unwrap(), path)
        }
    };

    if let Some(inode) = parent_inode.walk(&abs_path) {    
        return inode.do_open(
            Some(Arc::downgrade(&parent_inode)),
            flags,
            abs_path
        );
    }

    if flags.contains(OpenFlags::O_CREAT) {
        info!("[vfs open] create");
        return create_file(abs_path.clone(), parent_path, child_name, flags);
    }

    None
}

/// 创建一个新的文件夹
/// 
/// - path: 文件夹目录（绝对路径）
/// - mode: 创建模式
pub fn mkdir(path: &str, mode: usize) -> Option<FileClass> {
    // info!("open() abs_path is {}", path);

    // 查看当前路径是否是设备
    if find_device(path) {
        return None;
    }

    // 查看当前路径是否已经存在
    if INODE_CACHE.has_inode(path) {
        return None;
    }

    // 搜索上级文件夹
    // 获得上级文件夹文件路径
    let (parent_path, child_name) = Path::string2path(path.to_string()).split_with("/");
    // 获取上级文件夹的inode，等到创建inode的时候需要，如果上级文件夹的inode不存在就报错
    let (parent_inode, _) = if INODE_CACHE.has_inode(&parent_path) {
        (INODE_CACHE.get(&parent_path).unwrap(), "") // Get the parent inode if it exists
    } else {
        // If the parent inode does not exist, use the root inode
        if parent_path == "/" {
            (root_inode(), path)
        } else {
            (root_inode().walk(&parent_path).unwrap(), path)
        }
    };
    // 查看当前上级文件夹下是否有该文件，如果有该文件就返回错误
    if let Some(_) = parent_inode.walk(path) {
        return None;
    }
    // 利用parent_inode在根据绝对路径去创造新文件
    
    debug!(
        "[mkdir] path {}, mode {}",
        path, mode
    );
    
    return create_file(path.to_string(), &parent_path, &child_name, OpenFlags::O_DIRECTORY);

}
