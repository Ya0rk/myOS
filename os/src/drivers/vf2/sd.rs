use visionfive2_sd::{SDIo, SleepOps, Vf2SdDriver};

use crate::{drivers::{vf2::utils::{sleep_ms, sleep_ms_until}, BaseDriver, BlockDriver}, hal::config::{BLOCK_SIZE, GB, KERNEL_ADDR_OFFSET}, sync::SpinNoIrqLock};

pub struct Vf2BlkDev(SpinNoIrqLock<Vf2SdDriver<SdIoImpl, SleepOpsImpl>>);

impl Vf2BlkDev {
    pub fn new_and_init() -> Self {
        let mut vf2_driver = Vf2SdDriver::new(SdIoImpl);
        vf2_driver.init();
        Vf2BlkDev(SpinNoIrqLock::new(vf2_driver))
    }
}

impl BlockDriver for Vf2BlkDev {
    // sd空间为32G
    fn num_blocks(&self) -> usize {
        32 * GB / self.block_size()
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> crate::drivers::DevResult {
        self.0.lock().read_block(block_id, buf);
        Ok(())
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> crate::drivers::DevResult {
        self.0.lock().write_block(block_id, buf);
        Ok(())
    }

    fn flush(&self) -> crate::drivers::DevResult {
        todo!()
    }
}

impl BaseDriver for Vf2BlkDev {
    fn device_name(&self) -> &str {
        "vf2-sdio"
    }

    fn device_type(&self) -> crate::drivers::DeviceType {
        crate::drivers::DeviceType::Block
    }
}
pub struct SdIoImpl;
pub const SDIO_BASE: usize = 0x16020000;
pub const KERNEL_SDIO_BASE: usize = SDIO_BASE + KERNEL_ADDR_OFFSET;

impl SDIo for SdIoImpl {
    fn read_reg_at(&self, offset: usize) -> u32 {
        let addr = (KERNEL_SDIO_BASE + offset) as *mut u32;
        unsafe { addr.read_volatile() }
    }
    fn write_reg_at(&mut self, offset: usize, val: u32) {
        let addr = (KERNEL_SDIO_BASE + offset) as *mut u32;
        unsafe { addr.write_volatile(val) }
    }
    fn read_data_at(&self, offset: usize) -> u64 {
        let addr = (KERNEL_SDIO_BASE + offset) as *mut u64;
        unsafe { addr.read_volatile() }
    }
    fn write_data_at(&mut self, offset: usize, val: u64) {
        let addr = (KERNEL_SDIO_BASE + offset) as *mut u64;
        unsafe { addr.write_volatile(val) }
    }
}


pub struct SleepOpsImpl;

impl SleepOps for SleepOpsImpl {
    fn sleep_ms(ms: usize) {
        sleep_ms(ms)
    }
    fn sleep_ms_until(ms: usize, f: impl FnMut() -> bool) {
        sleep_ms_until(ms, f)
    }
}