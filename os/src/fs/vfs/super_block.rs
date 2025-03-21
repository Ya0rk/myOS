use alloc::sync::Arc;
use super::InodeTrait;

pub trait SuperBlockTrait: Send + Sync {
    /// 获取根节点
    fn root_inode(&self) -> Arc<dyn InodeTrait>;
    /// 将数据写回磁盘
    fn sync(&self);
    // fn fs_stat(&self) -> Statfs;
    /// 列出应用
    fn ls(&self);
}