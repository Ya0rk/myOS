use spin::Mutex;
use virtio_drivers::{device::blk::VirtIOBlk, transport::Transport, Hal};

use crate::{drivers::{BaseDriver, BlockDriver, DevResult, DeviceType}, sync::SpinNoIrqLock};

use super::as_dev_err;

pub struct VirtIoBlkDev<H: Hal, T: Transport> {
    inner: SpinNoIrqLock<VirtIOBlk<H, T>>,
}

unsafe impl<H: Hal,T: Transport> Send for VirtIoBlkDev<H, T> {}
unsafe impl<H: Hal,T: Transport> Sync for VirtIoBlkDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoBlkDev<H, T> {
    pub fn new(header: T) -> Self {
        Self {
            inner: SpinNoIrqLock::new(VirtIOBlk::<H, T>::new(header).expect("VirtIOBlk create failed")),
        }
    }
}

impl<H: Hal, T: Transport> BaseDriver for VirtIoBlkDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-blk"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
}

impl<H: Hal, T: Transport> BlockDriver for VirtIoBlkDev<H, T> {
    #[inline]
    fn num_blocks(&self) -> usize {
        self.inner.lock().capacity() as usize
    }

    #[inline]
    fn block_size(&self) -> usize {
        512
    }

    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult {
        self.inner
            .lock()
            .read_blocks(block_id as _, buf)
            .map_err(as_dev_err)
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult {
        self.inner
            .lock()
            .write_blocks(block_id as _, buf)
            .map_err(as_dev_err)
    }

    fn flush(&self) -> DevResult {
        Ok(())
    }
}
