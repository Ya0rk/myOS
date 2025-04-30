use alloc::sync::Arc;
use crate::fs::{ext4::Ext4Inode, Kstat};

use super::InodeTrait;

pub trait SuperBlockTrait: Send + Sync {
    /// 获取根节点
    fn root_inode(&self) -> Arc<Ext4Inode>;
    /// 将数据写回磁盘
    fn sync(&self);
    fn fs_stat(&self) -> Kstat;
    /// 列出应用
    fn ls(&self);
}