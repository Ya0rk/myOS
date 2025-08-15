use alloc::{collections::btree_map::BTreeMap, format, string::{String, ToString}, sync::Arc, vec::Vec};
use crate::{fs::{dirent::build_dirents, procfs::sys::fs::pipe_max_size::PipeMaxSizeInode, AbsPath, Dirent, InodeMeta, InodeTrait, InodeType, PageCache}, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;


pub struct FsDirInode {
    metadata: InodeMeta,
    pub children: BTreeMap<String, Arc<dyn InodeTrait>>,
}

impl FsDirInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        let mut children = BTreeMap::new();
        children.insert("pipe-max-size".to_string(), PipeMaxSizeInode::new());
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::Dir,
                0,
                "/proc/sys/fs".into(),
            ),
            children,
        })
    }
}

#[async_trait]
impl InodeTrait for FsDirInode {
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
        0
    }
    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
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
        let mut entries = alloc::vec![
            (".", 2, 4), 
            ("..", 1, 4), 
            ("pipe-max-size", 4, 8),
        ];
        Some(build_dirents(entries))
    }
}