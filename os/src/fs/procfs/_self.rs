use alloc::{collections::btree_map::BTreeMap, string::{String, ToString}, sync::Arc, vec::Vec, boxed::Box};
use async_trait::async_trait;
use crate::fs::{dirent::build_dirents, procfs::exe::ExeInode, AbsPath, Dirent, InodeMeta, InodeTrait, Kstat};


pub struct _SelfInode{
    pub inodeMeta: InodeMeta,
    pub children: BTreeMap<String, Arc<dyn InodeTrait>>,
}

impl _SelfInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        let mut children = BTreeMap::new();
        children.insert("exe".to_string(), ExeInode::new());
        Arc::new(Self{
            inodeMeta: InodeMeta::new(
                crate::fs::InodeType::Dir,
                0,
                "/proc/self".into(),
            ),
            children,
        })
    }
}

#[async_trait]
impl InodeTrait for _SelfInode {
    fn metadata(&self) -> &InodeMeta {
        &self.inodeMeta
    }
    fn look_up(&self,path: &str) -> Option<Arc<dyn InodeTrait> > {
        let binding = AbsPath::new(String::from(path)).get_filename();
        let pattern = binding.as_str();
        return self.children.get(pattern).cloned();
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = 16877;
        res.st_nlink = 1;
        res
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        let mut entries = alloc::vec![
            (".", 2, 4), 
            ("..", 1, 4), 
            ("exe", 4, 8),
        ];
        Some(build_dirents(entries))
    }
    fn get_size(&self) -> usize {
        4000
    }
}