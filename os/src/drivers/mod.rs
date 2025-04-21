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
use crate::config::KERNEL_ADDR_OFFSET;


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