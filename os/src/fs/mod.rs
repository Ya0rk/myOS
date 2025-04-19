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
pub mod pre_data;
pub mod ext4;
pub mod tmp;

use ext4::Ext4Inode;
pub use ext4::{root_inode,ls};
pub use ffi::{OpenFlags, UmountFlags, MountFlags};
use lwext4_rust::bindings::{O_CREAT, O_RDWR, O_TRUNC};
use lwext4_rust::{Ext4File, InodeTypes};
use page_cache::PageCache;
pub use path::{Path, path_test, join_path_2_absolute};
pub use dirent::Dirent;
pub use inode_cache::*;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
use sbi_spec::pmu::cache_event::NODE;
pub use stat::Kstat;
pub use vfs::*;
pub use stdio::{Stdin, Stdout};
pub use crate::mm::page::Page;
pub use pre_data::*;
use crate::mm::page::PageType;
use devfs::{find_device, open_device_file, register_device};
use ffi::{MOUNTS, MEMINFO, LOCALTIME, ADJTIME};
use ext4::file::NormalFile;
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

// os\src\fs\mod.rs

pub fn init() {
    // flush_preload().await;
    create_init_files();
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
        let mounts = unsafe { mountsinfo.as_bytes_mut() };
        let mut mountbuf = unsafe { core::slice::from_raw_parts_mut(
            mounts.as_mut_ptr(),
            mounts.len(),
        ) };
        let mountssize = mountsfile.write(mountbuf).await?;
        debug!("create /proc/mounts with {} sizes", mountssize);
    }
    //创建/proc/meminfo系统内存使用情况
    if let Some(FileClass::File(memfile)) =
        open("/proc", "meminfo", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut meminfo = String::from(MEMINFO);
        let mut membuf;
        unsafe {
            let mem = meminfo.as_bytes_mut();
            membuf = core::slice::from_raw_parts_mut(mem.as_mut_ptr(), mem.len());
        }
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
        let mut adjtimebuf;
        unsafe {
            let adj = adjtime.as_bytes_mut();
            adjtimebuf = core::slice::from_raw_parts_mut(adj.as_mut_ptr(), adj.len());
        }
        let adjtimesize = adjtimefile.write(adjtimebuf).await?;
        debug!("create /etc/adjtime with {} sizes", adjtimesize);
    }
    //创建./etc/localtime记录时区
    if let Some(FileClass::File(localtimefile)) =
        open("/etc", "localtime", OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let mut localtime = String::from(LOCALTIME);
        let mut localtimebuf;
        unsafe {
            let local = localtime.as_bytes_mut();
            localtimebuf = core::slice::from_raw_parts_mut(
                local.as_mut_ptr(),
                local.len(),
            );
        }
        let localtimesize = localtimefile.write(localtimebuf).await?;
        debug!("create /etc/localtime with {} sizes", localtimesize);
    }
    Ok(())
}

fn create_open_file(
    target_abs_path: &str,
    parent_path: &str,
    flags: OpenFlags,
) -> Option<FileClass> {
    println!(
        "[create_file],flags={:?},abs_path={},parent_path={}",
        flags, target_abs_path, parent_path
    );

    // 一定能找到,因为除了RootInode外都有父结点
    let parent_dir = match INODE_CACHE.get(parent_path) {
        Some(inode) => inode,
        None => Ext4Inode::new(parent_path, InodeTypes::EXT4_DE_DIR, None),
    };

    // 再看看能不能找到target，如果找到就返回
    let target_inode = match INODE_CACHE.get(target_abs_path) {
        Some(inode) => {
            info!("this inode in cache, path = {}", target_abs_path);
            inode
        },
        None => {
            info!("this inode not in cache, path = {}", target_abs_path);
            parent_dir.do_create(target_abs_path, flags.node_type()).expect("[create_open_file] don't get inode")
        },
    };

    let res = {
        let osinode = NormalFile::new(
            flags,
            Some(Arc::downgrade(&parent_dir)),
            target_inode,
            target_abs_path.to_string(),
        );
        FileClass::File(Arc::new(osinode))
    };

    Some(res)
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<FileClass> {
    open(&"/", path, flags)
}

pub fn open(cwd: &str, path: &str, flags: OpenFlags) -> Option<FileClass> {
    let abs_path = Path::string2path(
        join_path_2_absolute(
            cwd.to_string(), 
            path.to_string()
    ));

    if find_device(&abs_path.get()) {
        if let Some(device) = open_device_file(&abs_path.get()) {
            return Some(FileClass::Abs(device));
        }
        return None;
    }
    
    let create_inode_type = match flags.contains(OpenFlags::O_DIRECTORY) {
        true  => InodeTypes::EXT4_DE_DIR,
        false => InodeTypes::EXT4_DE_REG_FILE,
    };

    match flags.contains(OpenFlags::O_CREAT) {
        true => {
            let root = root_inode();
            let mut bind = root.file.lock();
            match bind.check_inode_exist(&abs_path.get(), create_inode_type.clone()) {
                true  => {
                    info!("path = {} is exitbbbbbbbbb", abs_path.get());
                    let parent_abs = abs_path.get_parent_abs();
                    return create_open_file(&abs_path.get(), &parent_abs, flags);
                },
                false => {
                    info!("path = {} no exitsssssssssss", abs_path.get());
                    // 说明在lwext4中还找不到这个inode
                    // 那么父母一定是存在,父母不存在返回错误
                    let parent_abs = abs_path.get_parent_abs();
                    // 利用lwext4创建ext4file
                    if create_inode_type == InodeTypes::EXT4_DE_DIR {
                        mkdir(path, 0);
                    }
                    let mut file = Ext4File::new(&abs_path.get(), create_inode_type.clone());
                    file.file_open(&abs_path.get(), O_RDWR | O_CREAT | O_TRUNC); // 为他创建lwext4的ext4inode
                    file.file_close();
                    
                    drop(bind);

                    let res = root_inode().file.lock().check_inode_exist(&abs_path.get(), create_inode_type.clone());
                    info!("now path = {}, exits = {}", abs_path.get(), res);

                    return create_open_file(&abs_path.get(), &parent_abs, flags);
                }
            }
        },
        false => {
            // 不用创建的话，说明文件存在，直接打开即可
            // 不存在lwext4中(代表unlink将其删掉了)同时又没有create flag，代表打开的文件不存在，直接返回none
            if !root_inode()
                .file
                .lock()
                .check_inode_exist(&abs_path.get(), create_inode_type) {
                    return None;
            }

            let parent_abs = abs_path.get_parent_abs();
            return create_open_file(&abs_path.get(), &parent_abs, flags);
        },
    }

    None
}

/// 创建一个新的文件夹
/// 
/// - path: 文件夹目录（绝对路径）
/// - mode: 创建模式
pub fn mkdir(target_abs_path: &str, mode: usize) -> SysResult<()> {
    // info!("open() abs_path is {}", path);
    // 查看当前路径是否是设备
    if find_device(target_abs_path) {
        return Err(Errno::EEXIST);
    }

    // 查看当前路径是否已经存在
    if INODE_CACHE.has_inode(target_abs_path) {
        return Err(Errno::EEXIST);
    }

    // 搜索上级文件夹
    // 获得上级文件夹文件路径
    let parent_abs = Path::string2path(target_abs_path.to_string()).get_parent_abs();
    // 获取上级文件夹的inode，等到创建inode的时候需要，如果上级文件夹的inode不存在就报错
    let parent_inode = if INODE_CACHE.has_inode(&parent_abs) {
        INODE_CACHE.get(&parent_abs).unwrap() // Get the parent inode if it exists
    } else {
        return Err(Errno::EEXIST);
    };
    // 查看当前上级文件夹下是否有该文件夹，如果有该文件夹就返回错误
    if parent_inode.walk(target_abs_path) {
        return Err(Errno::EEXIST);
    }
    // 利用parent_inode在根据绝对路径去创造新文件
    let mut parent_file = Ext4File::new(&parent_abs, InodeTypes::EXT4_DE_DIR);
    parent_file.dir_mk(target_abs_path);
    
    info!(
        "[mkdir] path {}, mode {}",
        target_abs_path, mode
    );
    Ok(())

}

pub fn chdir(target: &str) -> bool {
    info!("[chdir] target = {}", target);
    let bind = root_inode();
    let mut root = bind.file.lock();

    root.check_inode_exist(target, InodeTypes::EXT4_DE_DIR)
}