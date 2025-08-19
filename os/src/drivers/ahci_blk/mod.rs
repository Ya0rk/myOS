use core::cell::UnsafeCell;

use alloc::{sync::Arc, vec::Vec};

use crate::{
    drivers::{
        device::{dev_number::MajorNumber, BlockDevice, Device},
        tty::tty_core::CharDevice,
        BlockMajorNum, DevResult, DeviceType,
    },
    mm::FrameTracker,
};

pub struct Ahci_blk {
    major: MajorNumber,
    minor: usize,
    frames: UnsafeCell<Vec<FrameTracker>>,
}

unsafe impl Send for Ahci_blk {}
unsafe impl Sync for Ahci_blk {}

impl Device for Ahci_blk {
    fn get_type(&self) -> DeviceType {
        DeviceType::Block
    }
    fn get_major(&self) -> MajorNumber {
        self.major
    }
    fn get_minor(&self) -> usize {
        self.minor
    }
    fn as_char(self: Arc<Self>) -> Option<Arc<dyn CharDevice>> {
        None
    }
    // TODO: BlockDriver -> BlockDevice
    fn as_block(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
        Some(self)
    }
}

impl BlockDevice for Ahci_blk {
    fn num_blocks(&self) -> usize {
        todo!()
    }
    fn block_size(&self) -> usize {
        todo!()
    }
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult {
        todo!()
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult {
        todo!()
    }
    fn flush(&self) -> DevResult {
        todo!()
    }
}
