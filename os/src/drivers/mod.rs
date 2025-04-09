// pub mod block;

// pub use block::BLOCK_DEVICE;

mod device;
mod disk;
mod virtio;
pub mod loongarch;

pub use device::*;
pub use disk::*;
use log::info;
pub use virtio::*;

use virtio_drivers::{Hal, VirtIOHeader};

use crate::hal::config::KERNEL_ADDR_OFFSET;


pub type BlockDeviceImpl = VirtIoBlkDev<VirtIoHalImpl>;

impl BlockDeviceImpl {
    pub fn new_device() -> Self {
        info!("create a new BlockDeviceImpl device");
        let VIRTIO0: usize = crate::hal::config::VIRTIO0;
        unsafe { VirtIoBlkDev::new(&mut *(VIRTIO0 as *mut VirtIOHeader)) }
    }
}

use alloc::vec;
pub fn block_device_test() {
    let block_size: usize = 512; 
    info!("create a new BlockDeviceImpl test");
        let VIRTIO0: usize = crate::hal::config::VIRTIO0;
        let mut dev:VirtIoBlkDev<VirtIoHalImpl> = unsafe { VirtIoBlkDev::new(&mut *(VIRTIO0 as *mut VirtIOHeader))};
        let mut original_data = vec![0u8; block_size];
        let mut test_buffer = vec![0u8; block_size];
        let test_data = [0x55; 512];  // 测试数据模式：全部填充 0x55
        // 1. 读取原始数据
        dev.read_block(0, &mut original_data)
            .expect("Failed to read original data");

        // 2. 写入测试数据
        dev.write_block(0, &test_data)
            .expect("Failed to write test data");

        // 3. 读回并验证
        dev.read_block(0, &mut test_buffer)
            .expect("Failed to read back test data");

        // 4. 验证数据
        assert_eq!(&test_buffer[..], &test_data[..], "Read data doesn't match written data");

        // 5. 恢复原始数据
        dev.write_block(0, &original_data)
            .expect("Failed to restore original data");

        println!("Block device test passed!");

}