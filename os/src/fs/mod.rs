mod devfs;
mod dirent;
// mod inode_cache;
mod page_cache;
mod mount;
mod pipe;
mod stat;
mod stdio;
pub mod vfs;
mod path;
pub mod pre_data;
pub mod ext4;
// pub mod tmp;
pub mod ffi;

use core::error;

use ext4::{file, Ext4Inode};
pub use ext4::{root_inode,ls};
pub use ffi::*;
use lwext4_rust::bindings::{self, O_CREAT, O_RDWR, O_TRUNC};
use lwext4_rust::{Ext4File, InodeTypes};
use page_cache::PageCache;
pub use path::{Path, path_test, join_path_2_absolute};
pub use dirent::Dirent;
// pub use inode_cache::*;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
use sbi_rt::NonRetentive;
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
    // 应当初始化Dentry
    Dentry::init();
    create_init_files();
}

pub fn list_apps() -> bool{
    println!("/**** APPS ****");
    ls();
    println!("**************/");
    true
}

pub fn create_init_files() -> SysResult {
    //创建/proc文件夹
    mkdir("/proc", 0);
    let proc = Ext4Inode::new("/proc", InodeTypes::EXT4_DE_DIR, None);
    // 创建musl glibc文件夹
    // mkdir("/musl", 0);
    // let muslinode = Ext4Inode::new("/musl", InodeTypes::EXT4_DE_DIR, Some(PageCache::new_bare()));
    // mkdir("/glibc", 0);
    open_file("/musl/ls", OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    open_file("/ls", OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    mkdir("/bin", 0);
    // let glibcinode = Ext4Inode::new("/glibc", InodeTypes::EXT4_DE_DIR, Some(PageCache::new_bare()));
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
    }

    //创建/dev文件夹
    mkdir("/dev", 0);
    //注册设备/dev/rtc和/dev/rtc0
    register_device("/dev/rtc");
    register_device("/dev/rtc0");
    //注册设备/dev/tty
    register_device("/dev/tty");
    //注册设备/dev/zero
    register_device("/dev/zero");
    //注册设备/dev/numm
    register_device("/dev/null");
    
    register_device("/proc");

    //创建./dev/misc文件夹
    mkdir("/dev/misc", 0);
    //注册设备/dev/misc/rtc
    register_device("/dev/misc/rtc");

    //创建/etc文件夹
    mkdir("/etc", 0);
    if let Some(FileClass::File(file)) = open_file("/etc/passwd", OpenFlags::O_CREAT | OpenFlags::O_RDWR) {
        let buf = [0; 10];
        file.write(&buf);
    };
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
    }
    Ok(())
}

/// 创建一个打开的文件
/// 
/// target_abs_path: 应当为绝对路径
/// 
/// parent_path: 应当为绝对路径
fn create_open_file(
    target_abs_path: &str,
    parent_path: &str,
    flags: OpenFlags,
) -> Option<FileClass> {
    info!(
        "[create_file] flags={:?}, abs_path={}, parent_path={}",
        flags, target_abs_path, parent_path
    );

    // 逻辑为获得一个Option<Arc InodeTrait>如果返回None直接返回None,因为代表父母节点都没有
    // 如果父母节点存在, 那么当父母节点是Dir的时候获得inode,如果父母节点不是Dir页直接返回None
    let parent_dir = {
        if let Some(inode) = Dentry::get_inode_from_path(&(parent_path.into())) {
            if inode.node_type() == InodeType::Dir {
                inode
            } else {
                info!("[create_file] failed inode type is {:?}", inode.node_type());
                return None;
            }
        } else {
            return None;
        }
    };
    let parent_dentry = Dentry::get_dentry_from_path(&(parent_path.into())).unwrap();
    info!("[create_file] got parent inode");
    // 通过Dentry直接返回target_inode,如果节点存在就直接返回
    // 如果节点不存在就检查创建的标志位,
    // 如果需要创建就创建一个,使用InodeTrait::do_create方法
    // 如果不需要创建就直接返回None
    let target_inode =  {
        if let Some(inode) = Dentry::get_inode_from_path(&(target_abs_path.into())) {
            inode
        } else {
            if flags.contains(OpenFlags::O_CREAT) {
                // need to create
                let path = Path::string2path(String::from(target_abs_path));
                parent_dentry.add_child(&path.get_filename(), flags).unwrap()
            } else {
                // no need to create
                info!("[create_file] path = {} no need to creat", target_abs_path);
                return None;
            }
            
        }
    };
    info!("[create_file] got target inode");

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
    info!("[fs_open] cwd = {}, path = {}", cwd, path);
    let abs_path = Path::string2path(
        join_path_2_absolute(
            cwd.to_string(), 
            path.to_string()
    ));
    // info!("[open] abspath = {}", abs_path.get());

    // 临时保存这个机制,后期应当使用设备文件系统去代替
    if find_device(&abs_path.get()) {
        if let Some(device) = open_device_file(&abs_path.get()) {
            return Some(FileClass::Abs(device));
        }
        return None;
    }
    
    create_open_file(&abs_path.get(), &abs_path.get_parent_abs(), flags)
    
}

/// 创建一个新的文件夹
/// 
/// - path: 文件夹目录（绝对路径）
/// - mode: 创建模式
pub fn mkdir(target_abs_path: &str, mode: usize) -> SysResult<()> {
    info!("[mkdir] new dir abs_path is {}", target_abs_path);

    let abs_path = Path::string2path(target_abs_path.into());

    // 查看当前路径是否是设备
    if find_device(target_abs_path) {
        return Err(Errno::EEXIST);
    }

    
    info!(
        "[mkdir] path {}, mode {}",
        target_abs_path, mode
    );

    // 首先探测有没有这个文件,如果有就报错
    // 否则使用 OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT 去创建
    // 最后返回OK就可以
    if let Some(_) = Dentry::get_inode_from_path(&abs_path.get()) {
        return Err(Errno::EEXIST);
    } else {
        create_open_file(&abs_path.get(), &abs_path.get_parent_abs(), OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT);
    }

    Ok(())

}

pub fn chdir(target: &str) -> bool {
    info!("[chdir] target = {}", target);
    // let bind = root_inode();
    // let mut root = bind.file.lock();

    // root.check_inode_exist(target, InodeTypes::EXT4_DE_DIR)
    let path: String = target.into();
    if let Some(inode) = Dentry::get_inode_from_path(&path) {
        if inode.node_type() == InodeType::Dir {
            return true;
        }
    }
    false
}