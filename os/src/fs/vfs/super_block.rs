use crate::{fs::Kstat, syscall::StatFs};
use alloc::sync::Arc;

use super::InodeTrait;

/// SuperBlockTrait trait defines the interface for a superblock in a file system.
///
/// 超级块Trait 是文件系统的接口
///
/// 一个超级块类型代表一种文件系统实现
pub trait SuperBlockTrait: Send + Sync {
    /// 获取根节点
    fn root_inode(&self) -> Arc<dyn InodeTrait>;
    /// 将数据写回磁盘
    fn sync(&self);
    // 显示文件系统的信息
    fn fs_stat(&self) -> StatFs;
    /// 列出应用
    fn ls(&self);
}
