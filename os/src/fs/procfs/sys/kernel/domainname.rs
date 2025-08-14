use alloc::{format, string::String, sync::Arc, vec::Vec};
use crate::{fs::{InodeMeta, InodeTrait, InodeType, Kstat, ModeFlag, PageCache, StMode}, sync::SpinNoIrqLock, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;
use crate::info;


pub struct DomainNameInode {
    metadata: InodeMeta
}

impl DomainNameInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::File,
                0,
                "/proc/sys/kernel/domainname".into(),
            ),
        })
    }
}

#[async_trait]
impl InodeTrait for DomainNameInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }

    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        debug_point!("");
        DOMAINNAME.lock().write(buf)
    }

    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        // 疑似被弃用
        self.read_at(offset, buf).await
    }

    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        // 这里不能write_directly
        debug_point!("");
        DOMAINNAME.lock().write(buf)
    }

    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        debug_point!("");
        // let domainname = DOMAINNAME.lock().read();

        let bind = DOMAINNAME.lock();
        let meminfo = bind.read();
        let len = bind.len();
        if offset < len {
            let read_len = core::cmp::min(len - offset, buf.len());
            buf[..read_len].copy_from_slice(&meminfo[offset..offset + read_len]);
            read_len
        } else {
            0
        }
    }

    fn get_size(&self) -> usize {
        512
    }

    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = StMode::new(
            ModeFlag::S_IRUSR | ModeFlag::S_IRGRP | ModeFlag::S_IROTH | ModeFlag::S_IFREG).into();
        res.st_nlink = 1;
        res
    }
}

lazy_static! {
    pub static ref DOMAINNAME: SpinNoIrqLock<Domainname> =
        SpinNoIrqLock::new(Domainname::new("(domain)"));
}

// pub struct Domainname([u8; 256]);
pub struct Domainname {
    data: [u8; 64],
    size: usize,
}

impl Domainname {
    fn new(content: &str) -> Self {
        let mut res = Self {
            data: [0; 64],
            size: 0,
        };
        let data = content.as_bytes();
        let size = data.len();
        res.data[0..size].copy_from_slice(data);
        res.size = size;
        res
    }
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.data[0..buf.len()].copy_from_slice(buf);
        self.size = buf.len();
        // *self.0 = String::from_utf8(buf.to_vec()).ok().unwrap();
        buf.len()
    }
    pub fn read(&self) -> &[u8] {
        &self.data
    }
    pub fn len(&self) -> usize {
        self.size
    }
}