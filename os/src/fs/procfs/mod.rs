use alloc::sync::Arc;
use super_block::ProcFsSuperBlock;

pub mod inode;
pub mod super_block;

lazy_static! {
    /// procfs的超级块
    pub static ref PROCFS_SUPER_BLOCK: Arc<ProcFsSuperBlock> = Arc::new(ProcFsSuperBlock::new("/proc"));
}
