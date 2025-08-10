pub mod device;
pub mod device_new;
pub mod disk;
pub mod tty;
// pub mod dev_core;
#[cfg(feature = "2k1000la")]
pub mod k1000la;
pub mod loongarch_icu;
pub mod vf2;
pub mod virtio_driver;

use crate::{drivers::device_new::{dev_number::BlockMajorNum, manager::DEVICE_MANAGER, BlockDevice}, hal::config::KERNEL_ADDR_OFFSET};
use alloc::{sync::Arc, vec::Vec};
use core::any::Any;
use core::ptr::NonNull;
pub use device::*;
pub use disk::*;
use log::*;
use spin::RwLock;
pub use virtio_driver::*;
use virtio_drivers::{
    device::console::Size,
    transport::{
        self,
        mmio::{MmioTransport, VirtIOHeader},
        pci::PciTransport,
        Transport,
    },
};

use device::Device;

lazy_static::lazy_static! {
    pub static ref DEVICE_SET: RwLock<Vec<Device>> = RwLock::new(Vec::new());
}

pub fn register_block_device(dev: Arc<dyn BlockDriver>) {
    let device = Device::BlockDevice(dev);
    DEVICE_SET.write().push(device);
}

/// 获得一个任意一个块设备
pub fn get_block_device() -> Option<Arc<dyn BlockDevice>> {
    // let device_set = DEVICE_SET.read();
    // for dev in device_set.iter() {
    //     if let Device::BlockDevice(block_dev) = dev {
    //         return Some(block_dev.clone());
    //     }
    // }

    // None
    #[cfg(feature = "board_qemu")]
    let major = BlockMajorNum::VirtBlock;
    #[cfg(feature = "vf2")]
    let major = BlockMajorNum::MmcBlock;
    let minor = 0;
    DEVICE_MANAGER.read().get_block_dev(major, minor)
}

/// 设备的初始化
/// 会注册设备
/// 特别地在外部使用get_block_device去获得一个块设备
pub fn init(dtb_root: usize) {
    #[cfg(target_arch = "riscv64")]
    // let dt_root: usize = 0xffff_ffc0_bfe0_0000; //注意到应当看rustsbi的Device Tree Region信息
    let dt_root: usize = dtb_root + KERNEL_ADDR_OFFSET; // 上板的设备树地址
    #[cfg(target_arch = "loongarch64")]
    let dt_root: usize = 0x9000_0000_0010_0000;
    info!("satrt probe fdt tree root: {:X}", dt_root);
    // crate::drivers::virtio_driver::probe::probe(dt_root as u64);
    DEVICE_MANAGER.write().probe_initial(dt_root);
    #[cfg(target_arch = "riscv64")]
    crate::hal::rv64::arch::interrupt::plic_init();
    unsafe {
        crate::drivers::loongarch_icu::test_loongarch_icu(0x10000000 + KERNEL_ADDR_OFFSET);
    }
}

