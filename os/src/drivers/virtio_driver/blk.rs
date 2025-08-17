use core::ptr::NonNull;

use alloc::sync::Arc;
use spin::Mutex;
use virtio_drivers::{device::blk::VirtIOBlk, transport::{mmio::{MmioTransport, VirtIOHeader}, pci::{bus::{Cam, MmioCam, PciRoot}, virtio_device_type, PciTransport}, Transport}, Hal};
use virtio_drivers::transport::DeviceType as VirtIODeviceType;

use crate::{
    drivers::{
        device::{
            self, 
            dev_core::{PhysDriver, PhysDriverProbe}, 
            dev_number::{BlockMajorNum, MajorNumber}, 
            BlockDevice, Device
        }, 
        virtio_driver::pci::{allocate_bars, dump_bar_contents, PciDriverProbe, PciMemory32Allocator}, 
        DevResult, DeviceType, VirtIoHalImpl
    }, 
    hal::config::{DEVICE_ADDR_OFFSET, KERNEL_ADDR_OFFSET}, 
    sync::SpinNoIrqLock
};

use super::as_dev_err;

pub struct VirtIoBlkDev<H: Hal, T: Transport> {
    major: MajorNumber,
    minor: usize,
    inner: SpinNoIrqLock<VirtIOBlk<H, T>>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoBlkDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoBlkDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoBlkDev<H, T> {
    pub fn new(header: T, major: MajorNumber, minor: usize) -> Self {
        Self {
            major,
            minor,
            inner: SpinNoIrqLock::new(
                VirtIOBlk::<H, T>::new(header).expect("VirtIOBlk create failed"),
            ),
        }
    }
}

// impl<H: Hal, T: Transport> BaseDriver for VirtIoBlkDev<H, T> {
//     fn device_name(&self) -> &str {
//         "virtio-blk"
//     }

//     fn device_type(&self) -> DeviceType {
//         DeviceType::Block
//     }
// }

// impl<H: Hal, T: Transport> BlockDriver for VirtIoBlkDev<H, T> {
//     #[inline]
//     fn num_blocks(&self) -> usize {
//         self.inner.lock().capacity() as usize
//     }

//     #[inline]
//     fn block_size(&self) -> usize {
//         512
//     }

//     fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult {
//         self.inner
//             .lock()
//             .read_blocks(block_id as _, buf)
//             .map_err(as_dev_err)
//     }

//     fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult {
//         self.inner
//             .lock()
//             .write_blocks(block_id as _, buf)
//             .map_err(as_dev_err)
//     }

//     fn flush(&self) -> DevResult {
//         Ok(())
//     }
// }

impl<H: Hal + 'static, T: Transport + 'static> Device for VirtIoBlkDev<H, T> {
    fn get_type(&self) -> device::DeviceType {
        device::DeviceType::Block
    }

    fn get_major(&self) -> MajorNumber {
        // todo!()
        self.major
    }

    fn get_minor(&self) -> usize {
        // todo!()
        self.minor
    }
    fn as_block(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
        Some(self)
    }

}

impl<H: Hal + 'static, T: Transport + 'static> BlockDevice for VirtIoBlkDev<H, T> {
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


impl<H, T> PhysDriver for VirtIoBlkDev<H, T>
where
    H: Hal + 'static,
    T: Transport + 'static,
{
    fn irq_number(&self) -> Option<usize> {
        // TODO: 驱动缘故，virtio没实现中断
        None
    }
}

impl<'b, 'a, H> PhysDriverProbe<'b, 'a> for VirtIoBlkDev<H, MmioTransport<'static>>
where
    H: Hal + 'static,
//     T: Transport + 'static,
{
    fn probe(fdt: &'b flat_device_tree::Fdt<'a>) -> Option<Arc<Self>> {
        let virtio_nodes = fdt.all_nodes()
            .filter(| node | node.compatible().map(|c| c.all().any( |s| s == "virtio,mmio" )).unwrap_or(false) );
        for node in virtio_nodes {

            let mmio_range = node.reg().next()?;
            let base_va = mmio_range.starting_address as usize + KERNEL_ADDR_OFFSET;
            let header = NonNull::new(base_va as *mut VirtIOHeader).unwrap();

            if let Ok(transport) =  unsafe { MmioTransport::new(header, mmio_range.size.unwrap()) } 
                && transport.device_type() == VirtIODeviceType::Block {
                // let blk_driver = ;
                let blk_driver = Self::new(
                    transport, 
                    MajorNumber::Block(BlockMajorNum::VirtBlock),
                    0 // 不重复就行，只有一个设备
                ) ;
                println!("got one mmio");
                return Some(Arc::new(blk_driver));
                
                // return Self::new(transport).ok().map(Arc::new);
            }
        }
        None
    }
}

impl<'b, 'a, H> PhysDriverProbe<'b, 'a> for VirtIoBlkDev<H, PciTransport> 
where 
    H: Hal + 'static,
{
    fn probe(fdt: &'b flat_device_tree::Fdt<'a>) -> Option<Arc<Self>> {
        // todo!()
        if let Some(pci_node) = fdt.find_compatible(&["pci-host-ecam-generic"]) {
            log::info!("Found PCI node: {}", pci_node.name);
            Self::probe_pci(pci_node, Cam::Ecam)
        } else {
            None   
        }
        
    }
}


impl<'b, 'a, H> PciDriverProbe<'b, 'a> for VirtIoBlkDev<H, PciTransport> 
where 
    H: Hal + 'static,
    'a: 'b
{
    // adapted from drivers::virtio_driver::pci::enumerate_pci
    // TODO: refactor
    fn probe_pci(pci_node: flat_device_tree::node::FdtNode<'b, 'a>, cam: Cam) -> Option<Arc<Self>> {
        // todo!()
        let reg = pci_node.reg();
        let mut allocator = PciMemory32Allocator::for_pci_ranges(&pci_node);
        // info!("------show regs------");
        // for region in pci_node.reg() {
        //     info!(
        //         "Reg: {:?}-{:#x}",
        //         region.starting_address,
        //         region.starting_address as usize + region.size.unwrap()
        //     );
        // }
        // info!("------transport-------");
        for region in reg {
            // info!(
            //     "Reg: {:?}-{:#x}",
            //     region.starting_address,
            //     region.starting_address as usize + region.size.unwrap()
            // );

            // info!(
            //     "region size {:#X}, cam size {:#X}",
            //     region.size.unwrap(),
            //     cam.size() as usize
            // );
            // assert_eq!(region.size.unwrap(), cam.size() as usize);
            // SAFETY: We know the pointer is to a valid MMIO region.

            let mut pci_root = PciRoot::new(unsafe {
                MmioCam::new(
                    (region.starting_address as usize + DEVICE_ADDR_OFFSET) as *mut u8,
                    cam,
                )
            });

            for (device_function, info) in pci_root.enumerate_bus(0) {
                let (status, command) = pci_root.get_status_command(device_function);

                // info!(
                //     "Found {} at {} status: {:?} command {:?}\n",
                //     info, device_function, status, command
                // );

                if let Some(virtio_type) = virtio_device_type(&info) {
                    // info!("  VirtIO {:?}", virtio_type);
                    allocate_bars(&mut pci_root, device_function, &mut allocator);
                    dump_bar_contents(&mut pci_root, device_function, 4);
                    let mut transport =
                        PciTransport::new::<VirtIoHalImpl, _>(&mut pci_root, device_function).unwrap();
                    // info!(
                    //     "Detected virtio PCI device with device type {:?}, features {:#018x}",
                    //     transport.device_type(),
                    //     transport.read_device_features(),
                    // );
                    // info!("start transport");
                    // virtio_device(transport);
                    if virtio_type == VirtIODeviceType::Block {
                        let blk_driver = Self::new(
                            transport, 
                            MajorNumber::Block(BlockMajorNum::VirtBlock),
                            0 // 凑合用
                        ) ;
                        println!("got one pci");
                        return Some(Arc::new(blk_driver));
                        // return Self::new(transport).ok().map(Arc::new)
                    }
                    // info!("end transport");
                }
                // info!("should be next device");
            }
        }
        None
    }
}