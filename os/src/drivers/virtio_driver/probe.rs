//! os/src/driver/virtio_driver/probe.rs
//! 用于探测设备，将设备注册

use flat_device_tree::{node::FdtNode, standard_nodes::Compatible, Fdt};
use hashbrown::HashMap;
use log::info;
use crate::drivers::device::dev_number::{BlockMajorNum, MajorNumber};
use crate::drivers::{VirtIoBlkDev};
use crate::hal::config::KERNEL_ADDR_OFFSET;
use crate::sync::SpinNoIrqLock;
use alloc::sync::Arc;
use super::VirtIoHalImpl;
use core::ptr::NonNull;
use lazy_static::*;
use spin::RwLock;
use alloc::boxed::Box;
use spin::mutex::Mutex;
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
    Result,
};
#[cfg(feature = "vf2")]
use crate::drivers::vf2::Vf2SDIO;

lazy_static! {
    pub static ref BLOCKDEVICE_ADDR_REG: SpinNoIrqLock<Option<usize>> = SpinNoIrqLock::new(None);
}

lazy_static! {
    pub static ref BLOCKDEVICE_SIZE_REG: SpinNoIrqLock<Option<usize>> = SpinNoIrqLock::new(None);
}

// pub fn probe(fdt_ptr: u64) {
//     println!("fdt addr @{:X}", fdt_ptr);
//     let fdt = unsafe { Fdt::from_ptr(fdt_ptr as _).expect("fdt trans from ptr error") };
//     for node in fdt.all_nodes() {
//         println!(
//             "name: {} {:?}",
//             node.name,
//             node.compatible().map(Compatible::first),
//         );
//         for range in node.reg() {
//             println!(
//                 "   {:#018x?}, length {:?}",
//                 range.starting_address, range.size
//             )
//         }
//         if let (Some(compatible), Some(region)) = (node.compatible(), node.reg().next()) {
//             if compatible.all().any(|s| s == "virtio,mmio")
//                 && region.size.unwrap_or(0) > size_of::<VirtIOHeader>()
//             {
//                 println!("Found VirtIO MMIO device at {:?}", region);
//                 let size = region.size.unwrap();
//                 let addr = region.starting_address as usize + KERNEL_ADDR_OFFSET;
//                 let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
//                 println!(
//                     "addr: {:X} size: {:X} start trans to MmioTransport",
//                     addr, size
//                 );
//                 match unsafe { MmioTransport::new(header, size) } {
//                     Err(e) => println!("Error creating VirtIO MMIO transport: {:?}", e),
//                     Ok(transport) => {
//                         println!(
//                             "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
//                             transport.vendor_id(),
//                             transport.device_type(),
//                             transport.version(),
//                         );
//                         println!("check is it block");
//                         if transport.device_type() == DeviceType::Block {
//                             let mut addr_guard = BLOCKDEVICE_ADDR_REG.lock();
//                             *addr_guard = Some(addr);
//                             let mut size_guard = BLOCKDEVICE_SIZE_REG.lock();
//                             *size_guard = Some(size);
//                         }
//                         println!("finished check start to transport");
//                         virtio_device(transport);
//                     }
//                 }
//             }
//         }
//     }
//     /// 解析sd卡
//     #[cfg(feature = "vf2")]
//     if let Some(sdionode) = fdt.find_node("/soc/sdio1@16020000") {
//         probe_vf2sd(&sdionode);
//     }

//     #[cfg(target_arch = "loongarch64")]
//     if let Some(pci_node) = fdt.find_compatible(&["pci-host-cam-generic"]) {
//         log::info!("Found PCI node: {}", pci_node.name);
//         super::pci::enumerate_pci(pci_node, Cam::MmioCam);
//     }
//     #[cfg(target_arch = "loongarch64")]
//     if let Some(pcie_node) = fdt.find_compatible(&["pci-host-ecam-generic"]) {
//         log::info!("Found PCIe node: {}", pcie_node.name);
//         super::pci::enumerate_pci(pcie_node, Cam::Ecam);
//     }
// }

// #[cfg(feature = "vf2")]
// pub fn probe_vf2sd(sdionode: &FdtNode) {
//     // let sd_device = Vf2BlkDev::new_and_init(); // for package

//     // 获取寄存器信息
//     let base_address = sdionode.reg().next().unwrap().starting_address as usize;
//     let size = sdionode.reg().next().unwrap().size.unwrap();
//     // 获取中断号
//     let interrupt_number = match sdionode.interrupts().next() {
//         None => 33,
//         Some(a) => a,
//     };
//     // 创建 SDIO 设备
//     let sd_device = Vf2SDIO::new(base_address, size, interrupt_number, BlockMajorNum::MmcBlock, 0);
//     sd_device.card_init();
//     println!("[probe] find sd card, size = {}", sd_device.block_size() * sd_device.num_blocks());
//     register_block_device(Arc::new(sd_device));
// }

pub fn virtio_device(transport: impl Transport + 'static) {
    match transport.device_type() {
        DeviceType::Block => virtio_blk(transport),
        DeviceType::GPU => virtio_gpu(transport),
        DeviceType::Network => virtio_net(transport),
        DeviceType::Console => virtio_console(transport),
        DeviceType::Socket => match virtio_socket(transport) {
            Ok(()) => println!("virtio-socket test finished successfully"),
            Err(e) => println!("virtio-socket test finished with error {:?}", e),
        },
        t => println!("Unrecognized virtio device: {:?}", t),
    }
}

fn virtio_blk<T: Transport + 'static>(transport: T) {
    // let mut blk = VirtIOBlk::<VirtIoHalImpl, T>::new(transport).expect("failed to create blk driver");
    // println!("check blk readonly");
    // assert!(!blk.readonly());
    // println!("start to test blk");
    // let mut input = [0xffu8; 512];
    // let mut output = [0; 512];
    // for i in 0..32 {
    //     for x in input.iter_mut() {
    //         *x = i as u8;
    //     }
    //     blk.write_blocks(i, &input).expect("failed to write");
    //     blk.read_blocks(i, &mut output).expect("failed to read");
    //     assert_eq!(input, output);
    // }
    // println!("virtio-blk test finished");
    // info!("create a virtio block device");
    // let mut blk = Arc::new(VirtIoBlkDev::<VirtIoHalImpl, T>::new(transport, MajorNumber::Block(BlockMajorNum::VirtBlock), 0));
    // info!("register");
    // register_block_device(blk);
    // info!("finished register");
}

fn virtio_gpu<T: Transport>(transport: T) {
    // let mut gpu = VirtIOGpu::<VirtIoHalImpl, T>::new(transport).expect("failed to create gpu driver");
    // let (width, height) = gpu.resolution().expect("failed to get resolution");
    // let width = width as usize;
    // let height = height as usize;
    // println!("GPU resolution is {}x{}", width, height);
    // let fb = gpu.setup_framebuffer().expect("failed to get fb");
    // for y in 0..height {
    //     for x in 0..width {
    //         let idx = (y * width + x) * 4;
    //         fb[idx] = x as u8;
    //         fb[idx + 1] = y as u8;
    //         fb[idx + 2] = (x + y) as u8;
    //     }
    // }
    // gpu.flush().expect("failed to flush");
    // //delay some time
    // println!("virtio-gpu show graphics....");
    // for _ in 0..1000 {
    //     for _ in 0..100000 {
    //         unsafe {
    //             core::arch::asm!("nop");
    //         }
    //     }
    // }

    println!("virtio-gpu test finished");
}

fn virtio_net<T: Transport>(transport: T) {
    // let mut net =
    //     VirtIONetRaw::<VirtIoHalImpl, T, 16>::new(transport).expect("failed to create net driver");
    // let mut buf = [0u8; 2048];
    // let (hdr_len, pkt_len) = net.receive_wait(&mut buf).expect("failed to recv");
    // println!(
    //     "recv {} bytes: {:02x?}",
    //     pkt_len,
    //     &buf[hdr_len..hdr_len + pkt_len]
    // );
    // net.send(&buf[..hdr_len + pkt_len]).expect("failed to send");
    println!("virtio-net test finished");
}

fn virtio_console<T: Transport>(transport: T) {
    // let mut console =
    //     VirtIOConsole::<VirtIoHalImpl, T>::new(transport).expect("Failed to create console driver");
    // if let Some(size) = console.size().unwrap() {
    //     println!("VirtIO console {}", size);
    // }
    // for &c in b"Hello world on console!\n" {
    //     console.send(c).expect("Failed to send character");
    // }
    // let c = console.recv(true).expect("Failed to read from console");
    // println!("Read {:?}", c);
    println!("virtio-console test finished");
}

fn virtio_socket<T: Transport>(transport: T) -> virtio_drivers::Result<()> {
    // let mut socket = VsockConnectionManager::new(
    //     VirtIOSocket::<VirtIoHalImpl, T>::new(transport).expect("Failed to create socket driver"),
    // );
    // let port = 1221;
    // let host_address = VsockAddr {
    //     cid: VMADDR_CID_HOST,
    //     port,
    // };
    // println!("Connecting to host on port {port}...");
    // socket.connect(host_address, port)?;
    // let event = socket.wait_for_event()?;
    // assert_eq!(event.source, host_address);
    // assert_eq!(event.destination.port, port);
    // assert_eq!(event.event_type, VsockEventType::Connected);
    // println!("Connected to the host");

    // const EXCHANGE_NUM: usize = 2;
    // let messages = ["0-Ack. Hello from guest.", "1-Ack. Received again."];
    // for k in 0..EXCHANGE_NUM {
    //     let mut buffer = [0u8; 24];
    //     let socket_event = socket.wait_for_event()?;
    //     let VsockEventType::Received { length, .. } = socket_event.event_type else {
    //         panic!("Received unexpected socket event {:?}", socket_event);
    //     };
    //     let read_length = socket.recv(host_address, port, &mut buffer)?;
    //     assert_eq!(length, read_length);
    //     println!(
    //         "Received message: {:?}({:?}), len: {:?}",
    //         buffer,
    //         core::str::from_utf8(&buffer[..length]),
    //         length
    //     );

    //     let message = messages[k % messages.len()];
    //     socket.send(host_address, port, message.as_bytes())?;
    //     println!("Sent message: {:?}", message);
    // }
    // socket.shutdown(host_address, port)?;
    // println!("Shutdown the connection");
    Ok(())
}
