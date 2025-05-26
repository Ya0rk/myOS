use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use crate::{fs::{ffi::RenameFlags, Dirent, FileTrait, InodeTrait, InodeType, Kstat, OpenFlags, S_IFCHR}, mm::{page::Page, UserBuffer}, sync::{SpinNoIrqLock, TimeStamp}, task::get_init_proc, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;


pub struct DevTty;

impl DevTty {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevTty {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        let init_proc = get_init_proc();
        if let Some(tty_device) = init_proc.get_file_by_fd(0) {
            tty_device.read(user_buf).await
        } else {
            panic!("get Stdin error!");
        }
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        let init_proc = get_init_proc();
        if let Some(tty_device) = init_proc.get_file_by_fd(1) {
            tty_device.write(user_buf).await
        } else {
            panic!("get Stdout error!");
        }
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/tty".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        todo!()
    }
}


#[async_trait]
impl InodeTrait for DevTty {
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }

    fn do_create(&self, _path: &str, _ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn node_type(&self) -> InodeType {
        InodeType::CharDevice
    }

    async fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0
    }

    async fn write_at(&self, _offset: usize, _buf: &[u8]) -> usize {
        0
    }

    async fn write_directly(&self, _offset: usize, _buf: &[u8]) -> usize {
        0
    }

    fn truncate(&self, _size: usize) -> usize {
        0
    }

    async fn sync(&self) {}

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::new())
    }

    fn walk(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        stat
    }

    fn unlink(&self, _child_abs_path: &str) -> SysResult<usize> {
        Ok(0)
    }

    fn link(&self, _new_path: &str) -> SysResult<usize> {
        Err(crate::utils::Errno::EACCES)
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        unimplemented!("DevTty does not have a timestamp")
    }

    fn is_dir(&self) -> bool {
        false
    }

    // fn rename(&self, _old_path: &String, _new_path: &String) {}

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }

    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0
    }
}