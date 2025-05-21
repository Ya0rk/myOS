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
use virtio_drivers::{device::console::Size, transport::{self, mmio::{MmioTransport, VirtIOHeader}, pci::PciTransport, Transport}};
use core::ptr::NonNull;
use core::any::Any;
use log::*;
use alloc::{vec::Vec, sync::Arc};
use spin::RwLock;
use crate::hal::config::KERNEL_ADDR_OFFSET;


use device::Device;

lazy_static::lazy_static! {
    pub static ref DEVICE_SET: RwLock<Vec<Device>> = RwLock::new(Vec::new());
}

pub fn register_block_device(dev: Arc<dyn BlockDriver>) {
    let device = Device::BlockDevice(dev);
    DEVICE_SET.write().push(device);
}

/// 获得一个任意一个块设备
pub fn get_block_device() -> Option<Arc<dyn BlockDriver>> {
    let device_set = DEVICE_SET.read();
    for dev in device_set.iter() {
        if let Device::BlockDevice(block_dev) = dev {
            return Some(block_dev.clone());
        }
    }
    None
}

/// 设备的初始化
/// 会注册设备
/// 特别地在外部使用get_block_device去获得一个块设备
pub fn init() {
    #[cfg(target_arch = "riscv64")]
        let dt_root: usize = 0xffff_ffc0_bfe0_0000; //注意到应当看rustsbi的Device Tree Region信息
        #[cfg(target_arch = "loongarch64")]
        let dt_root: usize = 0x9000_0000_0010_0000;
        info!("satrt probe fdt tree root: {:X}", dt_root);
        crate::drivers::virtio_driver::probe::probe(dt_root as u64);
}

// #[cfg(target_arch = "riscv64")]
// pub type BlockDeviceImpl = VirtIoBlkDev<VirtIoHalImpl, MmioTransport<'static>>;
// #[cfg(target_arch = "loongarch64")]
// pub type BlockDeviceImpl = VirtIoBlkDev<VirtIoHalImpl, PciTransport<'static>>;


// impl BlockDeviceImpl {
//     pub fn new_device() -> Self {
//         let VIRTIO0: usize = virtio_driver::probe::BLOCKDEVICE_ADDR_REG.lock().unwrap();
//         let size: usize =  virtio_driver::probe::BLOCKDEVICE_SIZE_REG.lock().unwrap();
//         let header = NonNull::new(VIRTIO0 as *mut VirtIOHeader).unwrap();
//         match unsafe {
//             MmioTransport::new(header, size)
//         } {
//             Err(e) => {panic!("create virtio block device failed!")}
//             Ok(transport) => {
//                 info!(
//                     "Detected block device virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
//                     transport.vendor_id(),
//                     transport.device_type(),
//                     transport.version(),
//                 );
//                 unsafe { VirtIoBlkDev::new(transport) }
//             }
//         }
        
//     }
// }



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