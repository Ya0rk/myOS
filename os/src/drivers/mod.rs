// pub mod block;

// pub use block::BLOCK_DEVICE;

mod device;
mod disk;
// mod virtio;
pub mod virtio_driver;

pub use device::*;
pub use disk::*;
// pub use virtio::*;
pub use virtio_driver::*;
use virtio_drivers::{device::console::Size, transport::{self, mmio::{MmioTransport, VirtIOHeader}, Transport}};
use core::ptr::NonNull;
use log::*;
use crate::hal::config::KERNEL_ADDR_OFFSET;


pub type BlockDeviceImpl = VirtIoBlkDev<VirtIoHalImpl, MmioTransport<'static>>;

impl BlockDeviceImpl {
    pub fn new_device() -> Self {
        let VIRTIO0: usize = virtio_driver::probe::BLOCKDEVICE_ADDR_REG.lock().unwrap();
        let size: usize =  virtio_driver::probe::BLOCKDEVICE_SIZE_REG.lock().unwrap();
        let header = NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
        match unsafe {
            MmioTransport::new(header, size)
        } {
            Err(e) => {panic!("create virtio block device failed!")}
            Ok(transport) => {
                info!(
                    "Detected block device virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                );
                unsafe { VirtIoBlkDev::new(transport) }
            }
        }
        
    }
}

// use alloc::vec;
// pub fn block_device_test() {
//     let block_size: usize = 512; 
//     info!("create a new BlockDeviceImpl test");
//         let VIRTIO0: usize = crate::hal::config::VIRTIO0;
//         let mut dev:VirtIoBlkDev<VirtIoHalImpl> = unsafe { VirtIoBlkDev::new(&mut *(VIRTIO0 as *mut VirtIOHeader))};
//         let mut original_data = vec![0u8; block_size];
//         let mut test_buffer = vec![0u8; block_size];
//         let test_data = [0x55; 512];  // 测试数据模式：全部填充 0x55
//         // 1. 读取原始数据
//         dev.read_block(0, &mut original_data)
//             .expect("Failed to read original data");

//         // 2. 写入测试数据
//         dev.write_block(0, &test_data)
//             .expect("Failed to write test data");

//         // 3. 读回并验证
//         dev.read_block(0, &mut test_buffer)
//             .expect("Failed to read back test data");

//         // 4. 验证数据
//         assert_eq!(&test_buffer[..], &test_data[..], "Read data doesn't match written data");

//         // 5. 恢复原始数据
//         dev.write_block(0, &original_data)
//             .expect("Failed to restore original data");

//         println!("Block device test passed!");

// }