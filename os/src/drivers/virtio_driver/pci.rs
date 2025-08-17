use crate::{drivers::device::dev_core::{PhysDriver, PhysDriverProbe}, hal::config::DEVICE_ADDR_OFFSET};

use super::probe::virtio_device;
use super::VirtIoHalImpl;
use alloc::sync::Arc;
use flat_device_tree::{node::FdtNode, standard_nodes::Compatible, Fdt};
use log::info;
use zerocopy::IntoBytes;

use core::{
    mem::size_of,
    panic::PanicInfo,
    ptr::{self, NonNull},
};

use virtio_drivers::{
    device::{
        blk::VirtIOBlk,
        console::VirtIOConsole,
        gpu::VirtIOGpu,
        net::VirtIONetRaw,
        socket::{
            VirtIOSocket, VsockAddr, VsockConnectionManager, VsockEventType, VMADDR_CID_HOST,
        },
    },
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        pci::{
            bus::{
                BarInfo, Cam, Command, ConfigurationAccess, DeviceFunction, MemoryBarType, MmioCam,
                PciRoot,
            },
            virtio_device_type, PciTransport,
        },
        DeviceType, Transport,
    },
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PciRangeType {
    ConfigurationSpace,
    IoSpace,
    Memory32,
    Memory64,
}

impl From<u32> for PciRangeType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::ConfigurationSpace,
            1 => Self::IoSpace,
            2 => Self::Memory32,
            3 => Self::Memory64,
            _ => panic!("Tried to convert invalid range type {}", value),
        }
    }
}

pub fn enumerate_pci(pci_node: FdtNode, cam: Cam) {
    let reg = pci_node.reg();
    let mut allocator = PciMemory32Allocator::for_pci_ranges(&pci_node);
    info!("------show regs------");
    for region in pci_node.reg() {
        info!(
            "Reg: {:?}-{:#x}",
            region.starting_address,
            region.starting_address as usize + region.size.unwrap()
        );
    }
    info!("------transport-------");
    for region in reg {
        info!(
            "Reg: {:?}-{:#x}",
            region.starting_address,
            region.starting_address as usize + region.size.unwrap()
        );

        info!(
            "region size {:#X}, cam size {:#X}",
            region.size.unwrap(),
            cam.size() as usize
        );
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

            info!(
                "Found {} at {} status: {:?} command {:?}\n",
                info, device_function, status, command
            );

            if let Some(virtio_type) = virtio_device_type(&info) {
                info!("  VirtIO {:?}", virtio_type);
                allocate_bars(&mut pci_root, device_function, &mut allocator);
                dump_bar_contents(&mut pci_root, device_function, 4);
                let mut transport =
                    PciTransport::new::<VirtIoHalImpl, _>(&mut pci_root, device_function).unwrap();
                info!(
                    "Detected virtio PCI device with device type {:?}, features {:#018x}",
                    transport.device_type(),
                    transport.read_device_features(),
                );
                info!("start transport");
                virtio_device(transport);
                info!("end transport");
            }
            info!("should be next device");
        }
    }
}

pub trait PciDriverProbe<'b, 'a: 'b>: PhysDriverProbe<'b, 'a> {
    fn probe_pci(pci_node: FdtNode<'b, 'a>, cam: Cam) -> Option<Arc<Self>>;
}

/// Allocates 32-bit memory addresses for PCI BARs.
pub struct PciMemory32Allocator {
    start: u32,
    end: u32,
}

impl PciMemory32Allocator {
    /// Creates a new allocator based on the ranges property of the given PCI node.
    pub fn for_pci_ranges(pci_node: &FdtNode) -> Self {
        let mut memory_32_address = 0;
        let mut memory_32_size = 0;
        for range in pci_node.ranges() {
            let prefetchable = range.child_bus_address_hi & 0x4000_0000 != 0;
            let range_type = PciRangeType::from((range.child_bus_address_hi & 0x0300_0000) >> 24);
            let bus_address = range.child_bus_address as u64;
            let cpu_physical = range.parent_bus_address as u64;
            let size = range.size as u64;
            info!(
                "range: {:?} {}prefetchable bus address: {:#018x} host physical address: {:#018x} size: {:#018x}",
                range_type,
                if prefetchable { "" } else { "non-" },
                bus_address,
                cpu_physical,
                size,
            );
            // Use the largest range within the 32-bit address space for 32-bit memory, even if it
            // is marked as a 64-bit range. This is necessary because crosvm doesn't currently
            // provide any 32-bit ranges.
            if !prefetchable
                && matches!(range_type, PciRangeType::Memory32 | PciRangeType::Memory64)
                && size > memory_32_size.into()
                && bus_address + size < u32::MAX.into()
            {
                assert_eq!(bus_address, cpu_physical);
                memory_32_address = u32::try_from(cpu_physical).unwrap();
                memory_32_size = u32::try_from(size).unwrap();
            }
        }
        if memory_32_size == 0 {
            panic!("No 32-bit PCI memory region found.");
        }
        info!(
            "Using 32-bit PCI memory region: {:#x}-{:#x}",
            memory_32_address,
            memory_32_address + memory_32_size
        );
        Self {
            start: memory_32_address,
            end: memory_32_address + memory_32_size,
        }
    }

    /// Allocates a 32-bit memory address region for a PCI BAR of the given power-of-2 size.
    ///
    /// It will have alignment matching the size. The size must be a power of 2.
    pub fn allocate_memory_32(&mut self, size: u32) -> u32 {
        assert!(size.is_power_of_two());
        let allocated_address = align_up(self.start, size);
        assert!(allocated_address + size <= self.end);
        self.start = allocated_address + size;
        allocated_address
    }
}

const fn align_up(value: u32, alignment: u32) -> u32 {
    ((value - 1) | (alignment - 1)) + 1
}

pub fn dump_bar_contents(
    root: &mut PciRoot<impl ConfigurationAccess>,
    device_function: DeviceFunction,
    bar_index: u8,
) {
    let bar_info = root.bar_info(device_function, bar_index).unwrap();
    info!("Dumping bar {}: {:#x?}", bar_index, bar_info);
    if let Some(BarInfo::Memory { address, size, .. }) = bar_info {
        let start = address as *const u8;
        unsafe {
            let mut buf = [0u8; 32];
            for i in 0..size / 32 {
                let ptr = start.add(i as usize * 32);
                ptr::copy(ptr, buf.as_mut_ptr(), 32);
                if buf.iter().any(|b| *b != 0xff) {
                    // info!("  {:?}: {:x?}", ptr, buf);
                }
            }
        }
    }
    info!("End of dump");
}

/// Allocates appropriately-sized memory regions and assigns them to the device's BARs.
pub fn allocate_bars(
    root: &mut PciRoot<impl ConfigurationAccess>,
    device_function: DeviceFunction,
    allocator: &mut PciMemory32Allocator,
) {
    for (bar_index, info) in root.bars(device_function).unwrap().into_iter().enumerate() {
        let Some(info) = info else { continue };
        info!("BAR {}: {}", bar_index, info);
        // Ignore I/O bars, as they aren't required for the VirtIO driver.
        if let BarInfo::Memory {
            address_type, size, ..
        } = info
        {
            // For now, only attempt to allocate 32-bit memory regions.
            if size > u32::MAX.into() {
                info!("Skipping BAR {} with size {:#x}", bar_index, size);
                continue;
            }
            let size = size as u32;

            info!("device function is {:?}", device_function);

            match address_type {
                MemoryBarType::Width32 => {
                    if size > 0 {
                        let address = allocator.allocate_memory_32(size);
                        info!("Allocated address {:#010x}", address);
                        root.set_bar_32(device_function, bar_index as u8, address);
                    }
                }
                MemoryBarType::Width64 => {
                    if size > 0 {
                        let address = allocator.allocate_memory_32(size) as u64;
                        info!("Allocated address {:#010x}", address);
                        root.set_bar_64(device_function, bar_index as u8, address);
                    }
                }

                _ => panic!("Memory BAR address type {:?} not supported.", address_type),
            }
        }
    }

    // Enable the device to use its BARs.
    root.set_command(
        device_function,
        Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
    );
    let (status, command) = root.get_status_command(device_function);
    info!(
        "Allocated BARs and enabled device, status {:?} command {:?}",
        status, command
    );
}
