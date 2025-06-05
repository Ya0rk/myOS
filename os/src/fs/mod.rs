mod devfs;
mod dirent;
// mod inode_cache;
mod page_cache;
mod mount;
mod pipe;
mod stat;
mod stdio;
mod path;
pub mod vfs;
pub mod pre_data;
pub mod ext4;
pub mod procfs;
// pub mod tmp;
pub mod ffi;

use core::error;
use ext4::{file, Ext4Inode};
pub use ext4::{root_inode,ls};
pub use ffi::*;
use lwext4_rust::bindings::{self, O_CREAT, O_RDWR, O_TRUNC};
use lwext4_rust::{Ext4File, InodeTypes};
use page_cache::PageCache;
pub use path::{AbsPath, path_test, resolve_path};
pub use dirent::Dirent;
// pub use inode_cache::*;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
use procfs::PROCFS_SUPER_BLOCK;
use riscv::register;
// use sbi_rt::NonRetentive;
use sbi_spec::pmu::cache_event::NODE;
pub use stat::Kstat;
pub use stat::Statx;
pub use vfs::*;
pub use stdio::{Stdin, Stdout};
pub use crate::mm::page::Page;
use crate::net::dev;
pub use pre_data::*;
use crate::mm::page::PageType;
use devfs::{find_device, open_device_file, register_device, DevNull, DevZero};
use ffi::{MOUNTS, MEMINFO, LOCALTIME, ADJTIME};
use ext4::file::NormalFile;
use crate::mm::UserBuffer;
use crate::utils::{Errno, SysResult};
use alloc::string::{String, ToString};
use alloc::{sync::Arc, vec::Vec};
use log::{debug, error, info};

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

pub async fn init() {
    // 应当初始化Dentry
    println!("[Del0n1x] init fs start ...");
    Dentry::init();
    create_init_files().await;
}

pub async fn create_init_files() -> SysResult {
    mkdir("/usr".into(), 0);
    mkdir("/tmp".into(), 0);
    //创建/dev文件夹
    mkdir("/dev".into(), 0);
    //创建./dev/misc文件夹
    mkdir("/dev/misc".into(), 0);
    // libctest中的pthread_cancel_points测试用例需要
    mkdir("/dev/shm".into(), 0); 
    mkdir("/lib".into(), 0);
    mkdir("/bin".into(), 0);
    mkdir("/etc".into(), 0);
    if let Ok(FileClass::File(file)) = open("/etc/passwd".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR) {
        let buf = "nobody:x:0:0:nobody:/nonexistent:/usr/sbin/nologin\0".as_bytes();  // 这里是提前往里面写数据
        file.write(&buf).await;
    };
    
    //注册设备/dev/rtc和/dev/rtc0
    register_device("/dev/rtc");
    register_device("/dev/null");
    register_device("/dev/rtc0");
    //注册设备/dev/tty
    register_device("/dev/tty");
    //注册设备/dev/zero
    register_device("/dev/zero");
    register_device("/dev/loop0");
    //注册设备/dev/null
    // register_device("/dev/null");
    //注册设备/dev/misc/rtc
    register_device("/dev/misc/rtc");
    
    // open("/dev/null".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY);
    //创建/etc/adjtime记录时间偏差
    open("/etc/adjtime".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    //创建./etc/localtime记录时区
    open("/etc/localtime".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR);

    // 拷贝动态库
    dl_link("/musl/lib/libc.so", "/lib/ld-musl-riscv64-sf.so.1");
    dl_link("/musl/lib/libc.so", "/lib/ld-linux-riscv64-lp64d.so.1");
    dl_link("/musl/lib/libc.so", "/ld-linux-riscv64-lp64d.so.1");
    dl_link("/musl/lib/libc.so", "/lib/ld-musl-riscv64.so.1");
    dl_link("/musl/lib/dlopen_dso.so", "/musl/dlopen_dso.so");
    dl_link("/musl/lib/tls_get_new-dtv_dso.so", "/lib/tls_get_new-dtv_dso.so");

    Ok(())
}

fn dl_link(src: &str, target: &str) {
    if let Ok(inode) = Dentry::get_inode_from_path(src) {
        inode.link(&target.to_string());
    } else {
        panic!("no such dir: {}", src);
    }
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
) -> SysResult<FileClass> {
    info!(
        "    [create_open_file] flags={:?}, abs_path={}, parent_path={}",
        flags, target_abs_path, parent_path
    );

    // 逻辑为获得一个Option<Arc InodeTrait>如果返回None直接返回None,因为代表父母节点都没有
    // 如果父母节点存在, 那么当父母节点是Dir的时候获得inode,如果父母节点不是Dir页直接返回None
    if find_device(parent_path) {
        return Err(Errno::ENOTDIR);
    };
    let parent_dir = {
        Dentry::get_inode_from_path(parent_path)?
    };
    if parent_dir.node_type() != InodeType::Dir {
        info!("    [create_open_file] parent_path {} is not a directory", parent_path);
        return Err(Errno::ENOTDIR);
    }
    let parent_dentry = Dentry::get_dentry_from_path(parent_path).unwrap();
    // info!("[create_file] got parent inode");
    // 通过Dentry直接返回target_inode,如果节点存在就直接返回
    // 如果节点不存在就检查创建的标志位,
    // 如果需要创建就创建一个,使用InodeTrait::do_create方法
    // 如果不需要创建就直接返回None
    let target_inode =  {
        if let Ok(inode) = Dentry::get_inode_from_path(target_abs_path) {
            inode
        } else {
            if flags.contains(OpenFlags::O_CREAT) {
                // need to create
                let path: AbsPath = target_abs_path.into();
                parent_dentry.add_child(&path.get_filename(), flags).unwrap()
            } else {
                // no need to create
                debug!("[create_open_file] path = {} no need to creat", target_abs_path);
                error!("no create file = {}", target_abs_path);
                return Err(Errno::ENOENT);
            }
            
        }
    };

    if !target_inode.is_valid() {
        return Err(Errno::ENOENT);
    } 

    if flags.contains(OpenFlags::O_DIRECTORY) && target_inode.node_type() != InodeType::Dir {
        debug!("[create_open_file] target_path {} is not a directory", target_abs_path);
        return Err(Errno::ENOTDIR);
    }
    // info!("[create_file] got target inode");

    let res = {
        let osinode = NormalFile::new(
            flags,
            Some(Arc::downgrade(&parent_dir)),
            target_inode,
            target_abs_path.to_string(),
        );
        FileClass::File(Arc::new(osinode))
    };

    Ok(res)
}

/// path为绝对路径
pub fn open(path: AbsPath, flags: OpenFlags) -> SysResult<FileClass> {
    info!("    [fs_open] abspath = {}, flags = {:?}", path.get(), flags);
    debug_point!("    [open]");
    // info!("[open] abspath = {}", abs_path.get());
    if !path.is_absolute() {
        panic!("    [fs_open] path = {} is not absolte path.", path.get());
    }

    // 临时保存这个机制,后期应当使用设备文件系统去代替
    if find_device(&path.get()) {
        if let Some(device) = open_device_file(&path.get()) {
            return Ok(FileClass::Abs(device));
        }
        return Err(Errno::EIO);
    }
    
    create_open_file(&path.get(), &path.get_parent_abs(), flags)
}

/// 创建一个新的文件夹
/// 
/// - path: 文件夹目录（绝对路径）
/// - mode: 创建模式
pub fn mkdir(target_abs_path: AbsPath, mode: usize) -> SysResult<()> {
    debug!("[mkdir] new dir abs_path is {}", target_abs_path.get());

    // 查看当前路径是否是设备
    if find_device(&target_abs_path.get()) {
        return Err(Errno::EEXIST);
    }

    debug!(
        "[mkdir] path {}, mode {}",
        target_abs_path.get(), mode
    );
    debug_point!("[mkdir]");
    // 首先探测有没有这个文件,如果有就报错
    // 否则使用 OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT 去创建
    // 最后返回OK就可以
    if let Ok(_) = Dentry::get_inode_from_path(&target_abs_path.get()) {
        return Err(Errno::EEXIST);
    } else {
        create_open_file(&target_abs_path.get(), &target_abs_path.get_parent_abs(), OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT);
    }

    Ok(())

}

pub fn chdir(target: AbsPath) -> SysResult<()> {
    info!("[chdir] target = {}", target.get());
    
    let inode = Dentry::get_inode_from_path(&target.get())?;
    if inode.node_type() == InodeType::Dir {
        return Ok(());
    }
    return Err(Errno::ENOTDIR);
}