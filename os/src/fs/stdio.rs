use super::ffi::RenameFlags;
use super::FileTrait;
use super::InodeTrait;
use super::Kstat;
use super::OpenFlags;
use crate::hal::arch::console_getchar;
use crate::mm::{page::Page, UserBuffer};
use crate::task::get_current_hart_id;
use crate::utils::Errno;
use crate::utils::SysResult;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use lazy_static::lazy_static;
use spin::Mutex;

const LF: usize = 0x0a;
const CR: usize = 0x0d;

pub struct Stdin;

pub struct Stdout;

#[async_trait]
impl FileTrait for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_RDONLY
    }
    async fn read(&self, mut user_buf: &mut [u8]) -> SysResult<usize> {
        //一次读取多个字符
        let mut c: usize;
        let mut count: usize = 0;
        while count < user_buf.len() {
            c = console_getchar();
            if c > 255 {
                break;
            }
            user_buf[count] = c as u8;
            count += 1;
        }
        Ok(count)
    }
    async fn write(&self, _user_buf: &[u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
        // panic!("Cannot write to stdin!");
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("Stdin".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
    }

    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}

#[async_trait]
impl FileTrait for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_WRONLY
    }
    async fn read(&self, _user_buf: &mut [u8]) -> SysResult<usize> {
        panic!("Cannot read from stdout!");
    }
    async fn write_at(&self, offset: usize, buf: &[u8]) -> SysResult<usize> {
        self.write(buf).await
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        print!("{}", core::str::from_utf8(user_buf).unwrap());
        Ok(user_buf.len())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("Stdout".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}
