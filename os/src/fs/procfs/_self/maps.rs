use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use crate::{fs::{open, InodeMeta, InodeTrait, InodeType, Kstat, ModeFlag, OpenFlags, StMode}, utils::SysResult};
use crate::info;

pub struct MapsInode(pub InodeMeta);

impl MapsInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            InodeType::File,
            0,
            "/proc/self/maps".into(),
        )))
    }
}

#[async_trait]
impl InodeTrait for MapsInode {
    fn metadata(&self) ->  &InodeMeta {
        &self.0
    }

    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let maps = format!(
            r"555555554000-555555556000 r--p 00000000 00:42 5781                       /usr/bin/cat
555555556000-55555555a000 r-xp 00002000 00:42 5781                       /usr/bin/cat
55555555a000-55555555c000 r--p 00006000 00:42 5781                       /usr/bin/cat
55555555c000-55555555d000 r--p 00007000 00:42 5781                       /usr/bin/cat
55555555d000-55555555e000 rw-p 00008000 00:42 5781                       /usr/bin/cat
efffffff8000-f00008a76000 rw-p 00000000 00:00 0
ffff95bfe000-ffff95c00000 r--p 00000000 00:00 0                          [vvar]
ffff95c00000-ffff95c02000 r-xp 00000000 00:00 0                          [vdso]
ffffea742000-ffffea763000 rw-p 00000000 00:00 0                          [stack]"
        );
        let meminfo = Vec::from(maps);
        let len = meminfo.len();
        if offset < len {
            let read_len = core::cmp::min(len - offset, buf.len());
            buf[..read_len].copy_from_slice(&meminfo[offset..offset + read_len]);
            read_len
        } else {
            0
        }
    }

    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();

        res.st_mode = StMode::new(
            ModeFlag::S_IRUSR | ModeFlag::S_IRGRP | ModeFlag::S_IROTH | ModeFlag::S_IFREG).into();
        res.st_nlink = 1;
        res
    }
}