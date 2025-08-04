mod devfs;
mod dirent;
// mod inode_cache;
pub mod ext4;
mod mount;
mod page_cache;
mod path;
mod pipe;
pub mod pre_data;
pub mod procfs;
mod stat;
mod stdio;
pub mod vfs;
// pub mod tmp;
pub mod ffi;
pub mod ltp;
pub mod socketfs;

use core::error;
pub use dirent::Dirent;
use ext4::{file, Ext4Inode};
pub use ext4::{ls, root_inode};
pub use ffi::*;
use lwext4_rust::bindings::{self, true_, O_CREAT, O_RDWR, O_TRUNC};
use lwext4_rust::{Ext4File, InodeTypes};
// use page_cache::PageCache;
pub use path::{path_test, resolve_path, AbsPath};
// pub use inode_cache::*;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
use procfs::{inode, PROCFS_SUPER_BLOCK};
// use sbi_rt::NonRetentive;
pub use crate::mm::page::Page;
use crate::mm::page::PageType;
// use crate::mm::UserBuffer;
use crate::net::dev;
use crate::utils::{Errno, SysResult};
use alloc::string::{String, ToString};
use alloc::{sync::Arc, vec::Vec};
use devfs::{find_device, open_device_file, register_device, DevNull, DevZero};
use ext4::file::NormalFile;
use ffi::{MEMINFO, MOUNTS};
use log::{debug, error, info};
pub use page_cache::PageCache;
pub use pre_data::*;
use sbi_spec::pmu::cache_event::NODE;
pub use stat::Kstat;
pub use stat::Statx;
pub use stdio::{Stdin, Stdout};
pub use vfs::*;

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
    Dentry::init_dentry_sys();
    create_init_files().await;

    // Test case for file hole created by truncate and write
    println!("[fs test] start test file hole (truncate scenario)");
    let test_file_path = "/hole_test_truncate".into();
    if let Ok(FileClass::File(file)) = open(test_file_path, OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        // 1. Write initial data
        let initial_data = "initial data".as_bytes();
        file.write(initial_data).await.unwrap();
        println!("[fs test] wrote initial data");

        // 2. Truncate to 0
        let inode = file.get_inode();
        inode.truncate(0);
        println!("[fs test] truncated file to 0");

        // 3. Seek to a position > 0 to create a hole
        let hole_size = 10;
        file.lseek(hole_size as isize, SEEK_SET).unwrap();
        println!("[fs test] seeked to {}", hole_size);

        // 4. Write new data, creating a hole from 0 to 9
        let new_data = "new data".as_bytes();
        file.write(new_data).await.unwrap();
        println!("[fs test] wrote new data after hole");

        // 5. Seek back to the beginning to verify the hole
        file.lseek(0, SEEK_SET).unwrap();
        println!("[fs test] seeked to 0");

        // 6. Read from the hole and verify it's all zeros
        let mut hole_buf = [1u8; 10]; // Pre-fill with non-zero to be sure
        let read_len = file.read(&mut hole_buf).await.unwrap();
        assert_eq!(read_len, hole_size);
        for &byte in hole_buf.iter() {
            assert_eq!(byte, 0, "Byte in hole is not zero!");
        }
        println!("[fs test] hole content is verified to be zero");

        // 7. Verify the data written after the hole
        // The offset is now at the end of the hole (10)
        let mut data_buf = [0u8; 8];
        let read_len_data = file.read(&mut data_buf).await.unwrap();
        assert_eq!(read_len_data, new_data.len());
        assert_eq!(&data_buf[..read_len_data], new_data);
        println!("[fs test] data after hole is verified");

        println!("[fs test] file hole (truncate scenario) test pass");
    } else {
        println!("[fs test] file hole test fail: cannot create file");
    }
    // panic!("temp test");

    // Test cases for ksys_renameat2
    println!("[fs test] start comprehensive rename tests");
    use crate::hal::config::{AT_FDCWD, PAGE_SIZE, PATH_MAX, RLIMIT_NOFILE, USER_SPACE_TOP};
    use crate::syscall::fs::ksys_renameat2;

    // Helper function for renaming to reduce boilerplate
    fn do_rename(old_path_str: &str, new_path_str: &str) -> SysResult<usize> {
        let old_path = String::from(old_path_str);
        let new_path = String::from(new_path_str);
        ksys_renameat2(AT_FDCWD, old_path, AT_FDCWD, new_path, 0)
    }

    // Test Case 1: Rename a file and verify its content.
    println!("\n[fs test] start test 1: rename and verify content");
    let old_path_str1 = "/rename_content_old";
    let new_path_str1 = "/rename_content_new";
    let content1 = "hello world from rename test";

    // 1. Create and write to the old file
    if let Ok(FileClass::File(file)) = open(
        old_path_str1.into(),
        OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_TRUNC,
    ) {
        file.write(content1.as_bytes()).await.unwrap();
        println!("[fs test] created and wrote to {}", old_path_str1);
    } else {
        panic!("Failed to create {}", old_path_str1);
    }

    // // 2. Perform rename
    // assert!(
    //     do_rename(old_path_str1, new_path_str1).is_ok(),
    //     "rename failed"
    // );
    // println!("[fs test] renamed {} to {}", old_path_str1, new_path_str1);
    //
    // // 3. Verify old path is gone
    // assert!(
    //     matches!(
    //         open(old_path_str1.into(), OpenFlags::O_RDONLY),
    //         Err(Errno::ENOENT)
    //     ),
    //     "Old file should not exist"
    // );
    // println!("[fs test] verified old file does not exist");
    //
    // // 4. Verify new path exists and has correct content
    // if let Ok(FileClass::File(file)) = open(new_path_str1.into(), OpenFlags::O_RDONLY) {
    //     let mut buf = [0u8; 100];
    //     let len = file.read(&mut buf).await.unwrap();
    //     assert_eq!(
    //         &buf[..len],
    //         content1.as_bytes(),
    //         "Content mismatch after rename"
    //     );
    //     println!("[fs test] verified new file content is correct");
    // } else {
    //     panic!("Failed to open new file {}", new_path_str1);
    // }
    // println!("[fs test] PASS: rename and verify content");
    //
    // // Test Case 2: Rename to overwrite an existing file.
    // println!("\n[fs test] start test 2: rename to overwrite existing file");
    // let old_path_str2 = "/rename_overwrite_old";
    // let new_path_str2 = "/rename_overwrite_new";
    // let old_content2 = "old file content";
    // let new_content_initial2 = "new file initial content";
    //
    // // 1. Create old file with its content
    // if let Ok(FileClass::File(file)) = open(
    //     old_path_str2.into(),
    //     OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_TRUNC,
    // ) {
    //     file.write(old_content2.as_bytes()).await.unwrap();
    // } else {
    //     panic!("Failed to create {}", old_path_str2);
    // }
    //
    // // 2. Create new file with its own content
    // if let Ok(FileClass::File(file)) = open(
    //     new_path_str2.into(),
    //     OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_TRUNC,
    // ) {
    //     file.write(new_content_initial2.as_bytes()).await.unwrap();
    // } else {
    //     panic!("Failed to create {}", new_path_str2);
    // }
    //
    // // 3. Perform rename (overwrite)
    // assert!(
    //     do_rename(old_path_str2, new_path_str2).is_ok(),
    //     "rename (overwrite) failed"
    // );
    // println!(
    //     "[fs test] renamed (overwrite) {} to {}",
    //     old_path_str2, new_path_str2
    // );
    //
    // // 4. Verify old path is gone
    // assert!(
    //     matches!(
    //         open(old_path_str2.into(), OpenFlags::O_RDONLY),
    //         Err(Errno::ENOENT)
    //     ),
    //     "Old file should not exist after overwrite"
    // );
    //
    // // 5. Verify new path has the content of the old file
    // if let Ok(FileClass::File(file)) = open(new_path_str2.into(), OpenFlags::O_RDONLY) {
    //     let mut buf = [0u8; 100];
    //     let len = file.read(&mut buf).await.unwrap();
    //     assert_eq!(
    //         &buf[..len],
    //         old_content2.as_bytes(),
    //         "Content mismatch after overwrite"
    //     );
    //     println!("[fs test] verified overwritten file content is correct");
    // } else {
    //     panic!("Failed to open overwritten file {}", new_path_str2);
    // }
    // println!("[fs test] PASS: rename to overwrite existing file");

    // // Test Case 3: Rename a file into a different directory.
    // println!("\n[fs test] start test 3: rename into a subdirectory");
    // let dir_path3 = "/rename_test_dir";
    // let old_path_str3 = "/rename_subdir_old";
    // let new_path_str3 = "/rename_test_dir/rename_subdir_new";
    //
    // // 1. Create directory
    // mkdir(dir_path3.into(), 0).unwrap_or_else(|e| {
    //     if e != Errno::EEXIST {
    //         panic!("Failed to create directory: {:?}", e)
    //     }
    // });
    // println!("[fs test] created directory {}", dir_path3);
    //
    // // 2. Create the file to be moved
    // if let Ok(FileClass::File(file)) = open(
    //     old_path_str3.into(),
    //     OpenFlags::O_CREAT | OpenFlags::O_RDWR | OpenFlags::O_TRUNC,
    // ) {
    //     debug_point!("");
    //     file.write(b"move me").await.unwrap();
    // } else {
    //     panic!("Failed to create {}", old_path_str3);
    // }
    //
    // // 3. Perform rename
    // assert!(
    //     do_rename(old_path_str3, new_path_str3).is_ok(),
    //     "rename (into subdir) failed"
    // );
    // println!("[fs test] renamed {} to {}", old_path_str3, new_path_str3);
    //
    // // 4. Verify old path is gone
    // assert!(
    //     matches!(
    //         open(old_path_str3.into(), OpenFlags::O_RDONLY),
    //         Err(Errno::ENOENT)
    //     ),
    //     "Old file should not exist after move"
    // );
    //
    // // 5. Verify new path exists in the directory
    // assert!(
    //     open(new_path_str3.into(), OpenFlags::O_RDONLY).is_ok(),
    //     "New file does not exist in subdirectory"
    // );
    // println!("[fs test] verified file exists in new directory");
    // println!("[fs test] PASS: rename into a subdirectory");
    //
    // // INFO: 暂时对这个测试不作要求
    //
    // // Test Case 4: Rename a directory.
    // println!("\n[fs test] start test 4: rename a directory");
    // let old_dir_path4 = "/rename_dir_old";
    // let new_dir_path4 = "/rename_dir_new";
    // let file_inside_path_old4 = "/rename_dir_old/file_inside";
    // let file_inside_path_new4 = "/rename_dir_new/file_inside";
    //
    // // 1. Create old directory and a file inside it
    // mkdir(old_dir_path4.into(), 0).unwrap_or_else(|e| {
    //     if e != Errno::EEXIST {
    //         panic!("Failed to create directory: {:?}", e)
    //     }
    // });
    // println!("[fs test] created old directory and a file inside");
    //
    // // 2. Perform directory rename
    // assert!(
    //     do_rename(old_dir_path4, new_dir_path4).is_ok(),
    //     "rename (directory) failed"
    // );
    // println!(
    //     "[fs test] renamed directory {} to {}",
    //     old_dir_path4, new_dir_path4
    // );
    //
    // // 3. Verify old directory is gone
    // assert!(
    //     matches!(
    //         open(old_dir_path4.into(), OpenFlags::O_RDONLY),
    //         Err(Errno::ENOENT)
    //     ),
    //     "Old directory should not exist"
    // );
    //
    // // 4. Verify new directory exists and its content (the file) is there
    // let new_dir_inode =
    //     Dentry::get_inode_from_path(new_dir_path4).expect("New directory inode not found");
    // assert_eq!(
    //     new_dir_inode.node_type(),
    //     InodeType::Dir,
    //     "New path is not a directory"
    // );
    //
    // if let Ok(FileClass::File(file)) = open(file_inside_path_new4.into(), OpenFlags::O_RDONLY) {
    //     let mut buf = [0u8; 100];
    //     let len = file.read(&mut buf).await.unwrap();
    //     assert_eq!(
    //         &buf[..len],
    //         b"i am inside",
    //         "Content of file in renamed directory mismatch"
    //     );
    //     println!("[fs test] verified file inside renamed directory");
    // } else {
    //     panic!(
    //         "Failed to open file in new directory {}",
    //         file_inside_path_new4
    //     );
    // }
    // println!("[fs test] PASS: rename a directory");
    //
    // println!("\n[fs test] All rename tests finished successfully!");
    // panic!();
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
    mkdir("/lib64".into(), 0);
    mkdir("/bin".into(), 0);
    mkdir("/etc".into(), 0);
    if let Ok(FileClass::File(file)) =
        open("/etc/passwd".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR)
    {
        let buf = "nobody:x:0:0:nobody:/nonexistent:/usr/sbin/nologin\0".as_bytes(); // 这里是提前往里面写数据
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
    register_device("/dev/urandom");
    //注册设备/dev/null
    // register_device("/dev/null");
    //注册设备/dev/misc/rtc
    register_device("/dev/misc/rtc");

    if cfg!(feature = "autorun") {
        open("/bin/ls".into(), OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    }
    //创建/etc/adjtime记录时间偏差
    open(
        "/etc/adjtime".into(),
        OpenFlags::O_CREAT | OpenFlags::O_RDWR,
    );
    //创建./etc/localtime记录时区
    if let Ok(FileClass::File(file)) = open(
        "/etc/localtime".into(),
        OpenFlags::O_CREAT | OpenFlags::O_RDWR,
    ) {
        let buf = "/etc/localtime  Fri Jul 19 12:34:56 2024 CST\0".as_bytes(); // 这里是提前往里面写数据
        file.write(&buf).await;
    };

    if let Ok(FileClass::File(file)) = open(
        "/ltp_testcode_musl.sh".into(),
        OpenFlags::O_CREAT | OpenFlags::O_RDWR,
    ) {
        let buf = ltp::MUSL_LTP_testcode.as_bytes();
        file.write(&buf).await;
    }

    if let Ok(FileClass::File(file)) = open(
        "/ltp_testcode_glibc.sh".into(),
        OpenFlags::O_CREAT | OpenFlags::O_RDWR,
    ) {
        let buf = ltp::GLIBC_LTP_testcode.as_bytes();
        let buf = ltp::GLIBC_LTP_testcode.as_bytes();
        file.write(&buf).await;
    }

    Ok(())
}

fn dl_link(src: &str, target: &str) -> SysResult<usize> {
    let inode = Dentry::get_inode_from_path(src)?;
    let path: AbsPath = target.into();
    let parent = Dentry::get_dentry_from_path(&path.get_parent_abs())?;
    let new_path = parent
        .bare_child(&path.get_filename())
        .ok_or(Errno::EEXIST)?;
    info!("[dl_link]{} => {}", src, target);
    inode.link(new_path)
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
    let path = AbsPath::new(String::from(target_abs_path));
    let file_name = path.get_filename();

    // 逻辑为获得一个Option<Arc InodeTrait>如果返回None直接返回None,因为代表父母节点都没有
    // 如果父母节点存在, 那么当父母节点是Dir的时候获得inode,如果父母节点不是Dir页直接返回None
    if find_device(parent_path) {
        return Err(Errno::ENOTDIR);
    };
    let parent_dir = { Dentry::get_inode_from_path(parent_path)? };
    if parent_dir.node_type() != InodeType::Dir {
        info!(
            "    [create_open_file] parent_path {} is not a directory",
            parent_path
        );
        return Err(Errno::ENOTDIR);
    }
    let parent_dentry = Dentry::get_dentry_from_path(parent_path)?;
    debug_point!("");
    let target_inode = match flags.contains(OpenFlags::O_CREAT) {
        false => Dentry::get_inode_from_path(target_abs_path)?,
        true => {
            if let Ok(inode) = Dentry::get_inode_from_path(target_abs_path) {
                inode
            } else {
                debug_point!("");
                let bare_dentry = parent_dentry.bare_child(&file_name).unwrap();
                // 进行 do_create
                let res = {
                    if flags.contains(OpenFlags::O_DIRECTORY) {
                        parent_dir.do_create(bare_dentry, InodeType::Dir)
                    } else {
                        parent_dir.do_create(bare_dentry, InodeType::File)
                    }
                };
                // 判断 do_create 结果
                match res {
                    Some(inode) => {
                        debug_point!("");
                        inode
                    }
                    None => {
                        debug_point!("");
                        return Err(Errno::EIO);
                    }
                }
            }
        }
    };

    // if !target_inode.is_valid() {
    //     info!(
    //         "    [create_open_file] last check inode is no valid path: {}",
    //         target_abs_path
    //     );
    //     return Err(Errno::ENOENT);
    // }

    if flags.contains(OpenFlags::O_DIRECTORY) && target_inode.node_type() != InodeType::Dir {
        debug!(
            "[create_open_file] target_path {} is not a directory",
            target_abs_path
        );
        return Err(Errno::ENOTDIR);
    }
    info!("[create_file] got target inode, flags = {:?}", flags);

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
    info!(
        "    [fs_open] abspath = {}, flags = {:?}",
        path.get(),
        flags
    );
    // debug_point!("    [open]");
    // info!("[open] abspath = {}", abs_path.get());
    if !path.is_absolute() {
        // panic!("    [fs_open] path = {} is not absolte path.", path.get());
        return Err(Errno::ENOENT);
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

    debug!("[mkdir] path {}, mode {}", target_abs_path.get(), mode);
    debug_point!("[mkdir]");
    // 首先探测有没有这个文件,如果有就报错
    // 否则使用 OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT 去创建
    // 最后返回OK就可以
    if let Ok(_) = Dentry::get_inode_from_path(&target_abs_path.get()) {
        return Err(Errno::EEXIST);
    } else {
        create_open_file(
            &target_abs_path.get(),
            &target_abs_path.get_parent_abs(),
            OpenFlags::O_DIRECTORY | OpenFlags::O_CREAT,
        );
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
