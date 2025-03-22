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
pub mod ext4;
mod path;
mod file;

use devfs::{find_device, open_device_file, register_device};
pub use ext4::{root_inode,ls};
pub use ffi::{OpenFlags, UmountFlags, MountFlags};
use file::NormalFile;
use crate::mm::UserBuffer;
use crate::utils::{Errno, SysResult};
use alloc::string::{String, ToString};
use alloc::{sync::Arc, vec::Vec};
pub use path::{Path, path_test};
pub use dirent::Dirent;
pub use inode_cache::*;
use log::{debug, info};
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
pub use stat::Kstat;
pub use vfs::*;
pub use stdio::{Stdin, Stdout};

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
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InodeType {
    Unknown = 0o0,
    /// FIFO (named pipe)
    Fifo = 0o1,
    /// Character device
    CharDevice = 0o2,
    /// Directory
    Dir = 0o4,
    /// Block device
    BlockDevice = 0o6,
    /// Regular file
    File = 0o10,
    /// Symbolic link
    SymLink = 0o12,
    /// Socket
    Socket = 0o14,
}

impl InodeType {
    /// Tests whether this node type represents a regular file.
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }
    /// Tests whether this node type represents a directory.
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
    /// Tests whether this node type represents a symbolic link.
    pub const fn is_symlink(self) -> bool {
        matches!(self, Self::SymLink)
    }
    /// Returns `true` if this node type is a block device.
    pub const fn is_block_device(self) -> bool {
        matches!(self, Self::BlockDevice)
    }
    /// Returns `true` if this node type is a char device.
    pub const fn is_char_device(self) -> bool {
        matches!(self, Self::CharDevice)
    }
    /// Returns `true` if this node type is a fifo.
    pub const fn is_fifo(self) -> bool {
        matches!(self, Self::Fifo)
    }
    /// Returns `true` if this node type is a socket.
    pub const fn is_socket(self) -> bool {
        matches!(self, Self::Socket)
    }
}

core::arch::global_asm!(include_str!("preload.S"));

// os\src\fs\mod.rs
//将预加载到内存中的程序写入文件根目录
pub fn flush_preload() {
    extern "C" {
        fn initproc_start();
        fn initproc_end();
        // fn shell_start();
        // fn shell_end();
    }

    if let Some(FileClass::File(initproc)) = open_file("initproc", OpenFlags::O_CREAT) {
        let mut v = Vec::new();
        v.push(unsafe {
            core::slice::from_raw_parts_mut(
                initproc_start as *mut u8,
                initproc_end as usize - initproc_start as usize,
            ) as &'static mut [u8]
        });
        initproc.write(UserBuffer::new(v));
    }

    // if let Some(FileClass::File(onlinetests)) = open_file("onlinetests", OpenFlags::O_CREATE) {
    //     let mut v = Vec::new();
    //     v.push(unsafe {
    //         core::slice::from_raw_parts_mut(
    //             shell_start as *mut u8,
    //             shell_end as usize - shell_start as usize,
    //         ) as &'static mut [u8]
    //     });
    //     onlinetests.write(UserBuffer::new(v));
    // }
}

pub fn init() {
    INODE_CACHE.insert("/", root_inode());
    flush_preload();
    let _ = create_init_files();
}

pub fn list_apps() -> bool{
    println!("/**** APPS ****");
    ls();
    println!("**************/");
    true
}

//
const MOUNTS: &str = " ext4 / ext4 rw 0 0\n";
const MEMINFO: &str = r"
MemTotal:         944564 kB
MemFree:          835248 kB
MemAvailable:     873464 kB
Buffers:            6848 kB
Cached:            36684 kB
SwapCached:            0 kB
Active:            19032 kB
Inactive:          32676 kB
Active(anon):        128 kB
Inactive(anon):     8260 kB
Active(file):      18904 kB
Inactive(file):    24416 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:             0 kB
SwapFree:              0 kB
Dirty:                 0 kB
Writeback:             0 kB
AnonPages:          8172 kB
Mapped:            16376 kB
Shmem:               216 kB
KReclaimable:       9960 kB
Slab:              17868 kB
SReclaimable:       9960 kB
SUnreclaim:         7908 kB
KernelStack:        1072 kB
PageTables:          600 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:      472280 kB
Committed_AS:      64684 kB
VmallocTotal:   67108863 kB
VmallocUsed:       15740 kB
VmallocChunk:          0 kB
Percpu:              496 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
";
const ADJTIME: &str = "0.000000 0.000000 UTC\n";
const LOCALTIME: &str =
    "lrwxrwxrwx 1 root root 33 11月 18  2023 /etc/localtime -> /usr/share/zoneinfo/Asia/Shanghai\n";

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
    let (readable, writable) = flags.read_write();
    return parent_dir
        .create(&abs_path, flags.node_type())
        .map(|vfile| {
            INODE_CACHE.insert(&abs_path, vfile.clone());
            let osinode = NormalFile::new(
                readable,
                writable,
                vfile,
                Some(Arc::downgrade(&parent_dir)),
                abs_path,
            );
            FileClass::File(Arc::new(osinode))
        });
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<FileClass> {
    open(&"/", path, flags)
}

// pub fn open_dir(path: &str, flags: OpenFlags, mode: usize) -> Option<FileClass>{
//     unimplemented!()
// }

/// Opens a file or device at the specified path with the given flags.
///
/// # Arguments
///
/// * `cwd` - The current working directory.
/// * `path` - The path to the file to open.
/// * `flags` - The flags that determine how the file should be opened.
///
/// # Returns
///
/// An `Option<FileClass>` which is `Some(FileClass::File(vfile))` if the file is opened successfully,
/// or `Some(FileClass::Abs(device))` if a device file is opened, or `None` if the file cannot be opened.
pub fn open(cwd: &str, path: &str, flags: OpenFlags) -> Option<FileClass> {
    // Convert the provided path string into a Path object
    let kpath = Path::string2path(path.to_string());
    
    // Join the current working directory with the provided path to create an absolute path
    let new_path = kpath.join_path_2_absolute(cwd.to_string());
    let abs_path = new_path.get();
    info!("open() abs_path is {}", abs_path);
    // Check if the absolute path corresponds to a device file
    if find_device(&abs_path) {
        // Attempt to open the device file
        if let Some(device) = open_device_file(&abs_path) {
            return Some(FileClass::Abs(device)); // Return the opened device file
        }
        return None; // Return None if the device file cannot be opened
    }

    // Split the new path into parent directory and child file name
    let (parent_path, child_name) = new_path.split_with("/");
    let (parent_path, child_name) = (parent_path.as_str(), child_name.as_str());

    debug!(
        "[open] cwd={}, path={}, parent={}, child={}, abs={}",
        cwd, path, parent_path, child_name, &abs_path
    );

    // Check if the parent directory inode exists
    let (parent_inode, _) = if INODE_CACHE.has_inode(parent_path) {
        (INODE_CACHE.get(parent_path).unwrap(), child_name) // Get the parent inode if it exists
    } else {
        // If the parent inode does not exist, use the root inode
        if cwd == "/" {
            (root_inode(), path)
        } else {
            (root_inode().find_by_path(cwd).unwrap(), path)
        }
    };
    // 以上这里好像存在问题，这个所谓的find_by_path到底是什么含义

    // Attempt to find the inode for the specified absolute path
    if let Some(inode) = parent_inode.find_by_path(&abs_path) {
        // Insert the inode into the index for future reference
        INODE_CACHE.insert(&abs_path, inode.clone());
        
        // Determine if the file should be opened for reading or writing
        let (readable, writable) = flags.read_write();
        
        // Create a new OSInode instance for the file
        let vfile = NormalFile::new(
            readable,
            writable,
            inode,
            Some(Arc::downgrade(&parent_inode)), // Keep a weak reference to the parent inode
            abs_path,
        );

        // If the O_APPEND flag is set, move the file pointer to the end
        if flags.contains(OpenFlags::O_APPEND) {
            vfile.lseek(0, SEEK_END);
        }
        
        // If the O_TRUNC flag is set, truncate the file to zero length
        if flags.contains(OpenFlags::O_TRUNC) {
            vfile.inode.truncate(0);
        }
        
        // Return the opened file as a FileClass::File
        return Some(FileClass::File(Arc::new(vfile)));
    }

    // If the inode does not exist and the O_CREAT flag is set, create a new file
    if flags.contains(OpenFlags::O_CREAT) {
        return create_file(abs_path.clone(), parent_path, child_name, flags);
    }
    
    // Return None if the file cannot be opened or created
    None
}

/// 创建一个新的文件夹
/// 
/// - path: 文件夹目录（绝对路径）
/// - mode: 创建模式
pub fn mkdir(path: &str, mode: usize) -> Option<FileClass> {
    info!("open() abs_path is {}", path);

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
            (root_inode().find_by_path(&parent_path).unwrap(), path)
        }
    };
    // 查看当前上级文件夹下是否有该文件，如果有该文件就返回错误
    if let Some(_) = parent_inode.find_by_path(path) {
        return None;
    }
    // 利用parent_inode在根据绝对路径去创造新文件
    
    debug!(
        "[mkdir] path {}, mode {}",
        path, mode
    );
    
    return create_file(path.to_string(), &parent_path, &child_name, OpenFlags::O_DIRECTORY);

}
