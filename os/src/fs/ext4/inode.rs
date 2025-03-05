use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_SET},
    Ext4File, InodeTypes,
};

use crate::{
    fs::{Inode, InodeType, Kstat},
    sync::SyncUnsafeCell,
    utils::{GeneralRet, SysErrNo, SyscallRet},
};

use alloc::{string::String, sync::Arc, vec, vec::Vec};

pub struct Ext4Inode(SyncUnsafeCell<Ext4File>);

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    pub fn new(path: &str, types: InodeTypes) -> Self {
        Ext4Inode(SyncUnsafeCell::new(Ext4File::new(path, types)))
    }
}

impl Inode for Ext4Inode {
    fn size(&self) -> usize {
        let mut file = self.0.get_unchecked_mut();
        let types = as_inode_type(file.file_type_get());
        if types == InodeType::File {
            let path = file.get_path();
            let path = path.to_str().unwrap();
            file.file_open(path, O_RDONLY);
            let fsize = file.file_size();
            let _ = file.file_close();
            fsize as usize
        } else {
            0
        }
    }

    fn create(&self, path: &str, ty: InodeType) -> Option<Arc<dyn Inode>> {
        let types = as_ext4_de_type(ty);
        let file = self.0.get_unchecked_mut();
        let nf = Ext4Inode::new(path, types.clone());

        if !file.check_inode_exist(path, types.clone()) {
            let nfile = nf.0.get_unchecked_mut();
            if types == InodeTypes::EXT4_DE_DIR {
                if nfile.dir_mk(path).is_err() {
                    return None;
                }
            } else if nfile.file_open(path, O_RDWR | O_CREAT | O_TRUNC).is_err() {
                return None;
            } else {
                nfile.file_close();
            }
        }
        Some(Arc::new(nf))
    }

    fn node_type(&self) -> InodeType {
        as_inode_type(self.0.get_unchecked_mut().file_type_get())
    }

    fn read_at(&self, off: usize, buf: &mut [u8]) -> SyscallRet {
        let mut file = self.0.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| SysErrNo::EIO)?;
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| SysErrNo::EIO)?;
        let r = file.file_read(buf);
        let _ = file.file_close();
        r.map_err(|_| SysErrNo::EIO)
    }

    fn write_at(&self, off: usize, buf: &[u8]) -> SyscallRet {
        let mut file = self.0.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR).map_err(|_| SysErrNo::EIO)?;
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| SysErrNo::EIO)?;
        let r = file.file_write(buf);
        let _ = file.file_close();
        r.map_err(|_| SysErrNo::EIO)
    }

    fn truncate(&self, size: usize) -> GeneralRet {
        let mut file = self.0.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR | O_CREAT | O_TRUNC)
            .map_err(|_| SysErrNo::EIO)?;

        let t = file.file_truncate(size as u64);

        let _ = file.file_close();
        if let Err(_) = t {
            Err(SysErrNo::EIO)
        } else {
            Ok(())
        }
    }

    fn rename(&self, file: Arc<dyn Inode>) -> GeneralRet {
        todo!()
    }

    fn set_timestamps(&self, atime_sec: Option<u64>, mtime_sec: Option<u64>) -> GeneralRet {
        todo!()
    }

    fn sync(&self) {
        todo!()
    }

    fn read_all(&self) -> Result<Vec<u8>, SysErrNo> {
        let mut file = self.0.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| SysErrNo::EIO)?;
        let mut buf: Vec<u8> = vec![0; file.file_size() as usize];
        file.file_seek(0, SEEK_SET).map_err(|_| SysErrNo::EIO)?;
        let r = file.file_read(buf.as_mut_slice());
        let _ = file.file_close();
        if let Err(_) = r {
            Err(SysErrNo::EIO)
        } else {
            Ok(buf)
        }
    }

    fn find_by_path(&self, path: &str) -> Option<Arc<dyn Inode>> {
        let mut file = self.0.get_unchecked_mut();
        if file.check_inode_exist(path, InodeTypes::EXT4_DE_DIR) {
            // debug!("lookup new DIR FileWrapper");
            Some(Arc::new(Ext4Inode::new(path, InodeTypes::EXT4_DE_DIR)))
        } else if file.check_inode_exist(path, InodeTypes::EXT4_DE_REG_FILE) {
            // debug!("lookup new FILE FileWrapper");
            Some(Arc::new(Ext4Inode::new(path, InodeTypes::EXT4_DE_REG_FILE)))
        } else {
            None
        }
    }

    fn fstat(&self) -> Kstat {
        todo!()
    }

    fn read_dentry(&self, off: usize, len: usize) -> Option<(Vec<u8>, isize)> {
        todo!()
    }

    fn unlink(&self, child_name: &str) -> GeneralRet {
        todo!()
    }
}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        let mut file = self.0.get_unchecked_mut();
        // debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
    }
}

fn as_ext4_de_type(types: InodeType) -> InodeTypes {
    match types {
        InodeType::BlockDevice => InodeTypes::EXT4_DE_BLKDEV,
        InodeType::CharDevice => InodeTypes::EXT4_DE_CHRDEV,
        InodeType::Dir => InodeTypes::EXT4_DE_DIR,
        InodeType::Fifo => InodeTypes::EXT4_DE_FIFO,
        InodeType::File => InodeTypes::EXT4_DE_REG_FILE,
        InodeType::Socket => InodeTypes::EXT4_DE_SOCK,
        InodeType::SymLink => InodeTypes::EXT4_DE_SYMLINK,
        InodeType::Unknown => InodeTypes::EXT4_DE_UNKNOWN,
    }
}

fn as_inode_type(types: InodeTypes) -> InodeType {
    match types {
        InodeTypes::EXT4_INODE_MODE_FIFO => InodeType::Fifo,
        InodeTypes::EXT4_INODE_MODE_CHARDEV => InodeType::CharDevice,
        InodeTypes::EXT4_INODE_MODE_DIRECTORY => InodeType::Dir,
        InodeTypes::EXT4_INODE_MODE_BLOCKDEV => InodeType::BlockDevice,
        InodeTypes::EXT4_INODE_MODE_FILE => InodeType::File,
        InodeTypes::EXT4_INODE_MODE_SOFTLINK => InodeType::SymLink,
        InodeTypes::EXT4_INODE_MODE_SOCKET => InodeType::Socket,
        _ => {
            // warn!("unknown file type: {:?}", vtype);
            unreachable!()
        }
    }
}
