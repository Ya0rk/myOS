pub mod ahci_driver;

use ahci_driver::libahci::ahci_device;
use log::error;
use core::cell::UnsafeCell;

use alloc::{sync::Arc, vec::Vec};

use crate::{
    drivers::{
        ahci::ahci_driver::drv_ahci::{ahci_sata_read_common, ahci_sata_write_common}, device::{dev_core::{PhysDriver, PhysDriverProbe}, dev_number::MajorNumber, BlockDevice, Device}, tty::tty_core::CharDevice, BlockMajorNum, DevResult, DeviceType
    }, hal::BLOCK_SIZE, mm::FrameTracker, sync::SpinNoIrqLock
};

pub struct AhciBlock {
    major: MajorNumber,
    minor: usize,
    frames: UnsafeCell<Vec<FrameTracker>>,
    inner: SpinNoIrqLock<ahci_device>,
}

unsafe impl Send for AhciBlock {}
unsafe impl Sync for AhciBlock {}

impl Device for AhciBlock {
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

impl BlockDevice for AhciBlock {
    fn num_blocks(&self) -> usize {
        // todo!()
        let dev = self.inner.lock();
        let num = dev.blk_dev.lba as usize;
        // error!("[AhciBlock::num_blocks] num: {}", num);
        num
    }
    fn block_size(&self) -> usize {
        // todo!()
        // let dev = self.inner.lock();
        // dev.blk_dev.blksz as usize
        BLOCK_SIZE
    }
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult {
        // todo!()
        if buf.len() != BLOCK_SIZE {
            return Err(super::DevError::InvalidParam);
        }
        let dev = self.inner.lock();
        // dev.read_block(block_id, buf)
        let sector_size = dev.blk_dev.blksz as usize;
        let head_sector = block_id * BLOCK_SIZE / sector_size;
        let num_sector = 1 + (buf.len() - 1) / sector_size;
        // error!("[AhciBlock::read_block] block_id: {}, head_sector: {}, num_sector: {}, all_blocks: {}", block_id, head_sector, num_sector, self.num_blocks());
        // let res = 
        ahci_sata_read_common(&*dev, head_sector as u64, num_sector as u32, buf.as_mut_ptr());
        Ok(())
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult {
        // todo!()
         if buf.len() != BLOCK_SIZE {
            return Err(super::DevError::InvalidParam);
        }
        let dev = self.inner.lock();
        // dev.read_block(block_id, buf)
        let sector_size = dev.blk_dev.blksz as usize;
        let head_sector = block_id * BLOCK_SIZE / sector_size;
        let num_sector = 1 + (buf.len() - 1) / sector_size;
        // let res = 
        ahci_sata_write_common(&*dev, head_sector as u64, num_sector as u32, buf.as_ptr() as *mut u8);
        Ok(())
    }
    fn flush(&self) -> DevResult {
        todo!()
    }
}
impl AhciBlock {
    pub fn new() -> Self {
        let mut inner = ahci_device::default();
        use ahci_driver::drv_ahci::ahci_init;
        unsafe {
            let ret = ahci_init(&mut inner);
            error!("[AhciBlock::new] ahci_init ret: {}", ret);
        }
        Self {
            major: MajorNumber::Block(BlockMajorNum::MmcBlock),
            minor: 0,
            frames: UnsafeCell::new(Vec::new()),
            inner: SpinNoIrqLock::new(inner),
        }
    }
}

impl PhysDriver for AhciBlock {
    fn irq_number(&self) -> Option<usize> {
        // todo!()
        None
    }
}

impl<'b, 'a> PhysDriverProbe<'b, 'a> for AhciBlock {
    fn probe(fdt: &'b flat_device_tree::Fdt<'a>) -> Option<Arc<Self>> {
        // todo!()
        Some(Arc::new(AhciBlock::new()))
    }
}