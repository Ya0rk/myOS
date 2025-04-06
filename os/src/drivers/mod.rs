// pub mod block;

// pub use block::BLOCK_DEVICE;

mod device;
mod disk;
mod virtio;

pub use device::*;
pub use disk::*;
pub use virtio::*;

use virtio_drivers::VirtIOHeader;

use crate::hal::config::KERNEL_ADDR_OFFSET;


pub type BlockDeviceImpl = VirtIoBlkDev<VirtIoHalImpl>;

impl BlockDeviceImpl {
    pub fn new_device() -> Self {
        const VIRTIO0: usize = 0x10001000 + KERNEL_ADDR_OFFSET;
        unsafe { VirtIoBlkDev::new(&mut *(VIRTIO0 as *mut VirtIOHeader)) }
    }
}