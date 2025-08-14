use alloc::{collections::btree_map::BTreeMap, format, string::{String, ToString}, sync::Arc, vec::Vec};
use crate::{fs::{dirent::build_dirents, procfs::sys::{fs::FsDirInode, kernel::KernelDirInode}, AbsPath, Dirent, InodeMeta, InodeTrait, InodeType, PageCache}, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;


pub struct SysDirInode {
    metadata: InodeMeta,
    pub children: BTreeMap<String, Arc<dyn InodeTrait>>,
}

impl SysDirInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        let mut children = BTreeMap::new();
        children.insert("fs".to_string(), FsDirInode::new());
        children.insert("kernel".to_string(), KernelDirInode::new());
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::Dir,
                0,
                "/proc/sys".into(),
            ),
            children,
        })
    }
}

#[async_trait]
impl InodeTrait for SysDirInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }

    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        0
    }

    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // 非常重要
        // 这里不能write_at
        0
    }
    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        // 这里不能write_directly
        0
    }

    fn look_up(&self,path: &str) -> Option<Arc<dyn InodeTrait> > {
        let binding = AbsPath::new(String::from(path)).get_filename();
        let pattern = binding.as_str();
        return self.children.get(pattern).cloned();
    }

    fn get_size(&self) -> usize {
        512
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        let mut entries: Vec<(&str, u64, u8)> = alloc::vec![
            (".", 1, 4), 
            ("..", 0, 4), 
            ("fs", 2, 4), 
            ("kernel", 3, 4)
        ];
        Some(build_dirents(entries))
    }
}