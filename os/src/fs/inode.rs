use core::str::FromStr;

use super::{Dirent, File, Kstat};
use crate::drivers::BLOCK_DEVICE;
use crate::mm::UserBuffer;
use alloc::{string::String, sync::Arc};
use alloc::vec::Vec;
use bitflags::*;
use lazy_static::*;
use simple_fat32::{create_root_vfile, FAT32Manager, VFile, ATTR_ARCHIVE, ATTR_DIRECTORY};
use spin::Mutex;

/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}
/// The OS inode inner in 'Mutex'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<VFile>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(readable: bool, writable: bool, inode: Arc<VFile>) -> Self {
        Self {
            readable,
            writable,
            inner: Mutex::new(OSInodeInner { offset: 0, inode }),
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.lock();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
    
    pub fn is_dir(&self) -> bool {
        let inner = self.inner.lock();
        inner.inode.is_dir()
    }

    pub fn name(&self) -> String {
        let mut name = String::new();
        name.push_str(self.inner.lock().inode.name());
        name
    }

    pub fn delete(&self) -> usize {
        let inner = self.inner.lock();
        inner.inode.remove()
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<VFile> = {
        let efs = FAT32Manager::open(BLOCK_DEVICE.clone());
        Arc::new(create_root_vfile(&efs))
    };
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for (appname, appattr) in ROOT_INODE.ls().unwrap() {
        if appattr & ATTR_DIRECTORY == 0 {
            // 如果不是目录
            println!("{}", appname);
        }
    }
    println!("**************/");
}

bitflags! {
    #[derive(Clone, Copy)]
    ///Open file flags
    pub struct OpenFlags: u32 {
        ///Read only
        const O_RDONLY = 0;
        ///Write only
        const O_WRONLY = 1 << 0;
        ///Read & Write
        const O_RDWR = 1 << 1;
        /// set close_on_exec 
        const O_CLOEXEC = 1 << 7;
        ///Allow create
        const O_CREATE = 1 << 9;
        ///Clear file and return an empty one
        const O_TRUNC = 1 << 10;
        ///Directory
        const O_DIRECTROY = 1 << 21;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::O_WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    open("/", path, flags)
}

///Open file with flags
pub fn open(work_path: &str, path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let cur_inode = {
        if work_path == "/" {
            ROOT_INODE.clone()
        } else {
            let wpath: Vec<&str> = work_path.split('/').collect();
            ROOT_INODE.find_vfile_bypath(wpath).unwrap()
        }
    };

    let mut pathv: Vec<&str> = path.split('/').collect();

    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::O_CREATE) || flags.contains(OpenFlags::O_DIRECTROY) {
        if let Some(inode) = cur_inode.find_vfile_bypath(pathv.clone()) {
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            let mut create_type = 0;
            if flags.contains(OpenFlags::O_CREATE) {
                create_type = ATTR_ARCHIVE;
            } else if flags.contains(OpenFlags::O_DIRECTROY) {
                create_type = ATTR_DIRECTORY;
            }

            let name = pathv.pop().unwrap();

            if let Some(temp_inode) = cur_inode.find_vfile_bypath(pathv.clone()) {
                temp_inode
                    .create(name, create_type)
                    .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
            } else {
                None
            }
        }
    } else {
        cur_inode.find_vfile_bypath(pathv).map(|inode| {
            if flags.contains(OpenFlags::O_TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

// 更改当前工作路径
pub fn chdir(work_path: &str, path: &str) -> Option<String> {
    let current_inode = {
        if path.chars().nth(0).unwrap() == '/' {
            // 传入路径是绝对路径
            ROOT_INODE.clone()
        } else {
            // 传入路径是相对路径
            let current_work_pathv: Vec<&str> = work_path.split('/').collect();
            ROOT_INODE.find_vfile_bypath(current_work_pathv).unwrap()
        }
    };
    let pathv: Vec<&str> = path.split('/').collect();
    if let Some(_) = current_inode.find_vfile_bypath(pathv) {
        let new_current_path = String::from_str("/").unwrap() + &String::from_str(path).unwrap();
        if current_inode.name() == "/" {
            Some(new_current_path)
        } else {
            Some(String::from_str(current_inode.name()).unwrap() + &new_current_path)
        }
    } else {
        None
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn get_fstat(&self, kstat: &mut Kstat) {
        let inner = self.inner.lock();
        let vfile = inner.inode.clone();
        // todo
        let (st_size, st_blksize, st_blocks) = vfile.stat();
        kstat.init(st_size, st_blksize, st_blocks);
    }

    fn get_dirent(&self, dirent: &mut Dirent) -> isize {
        if !self.is_dir() {
            return -1;
        }
        let mut inner = self.inner.lock();
        let offset = inner.offset as u32;
        if let Some((name, off, _)) = inner.inode.dirent_info(offset as usize) {
            dirent.init(name.as_str());
            inner.offset = off as usize;
            let len = (name.len() + 8 * 4) as isize;
            len
        } else {
            -1
        }
    }

    fn get_name(&self) -> String {
        self.name()
    }

    fn set_offset(&self, offset: usize){
        let mut inner = self.inner.lock();
        inner.offset = offset;
    }
}
