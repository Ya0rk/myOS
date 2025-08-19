use core::char::MAX;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
#[macro_use]
use alloc::vec;
use embedded_hal::serial;
use flat_device_tree::Fdt;
// use riscv::register;
use spin::{rwlock::RwLock};
use virtio_drivers::{device::blk::VirtIOBlk, transport::{mmio::MmioTransport, pci::PciTransport}};

use crate::{
    drivers::{
        ahci::{self, AhciBlock}, device::{
            dev_core::{PhysDriver, PhysDriverProbe}, 
            dev_number::{BlockMajorNum, CharMajorNum, MajorNumber}, 
            irq::HardIrqHandler, BlockDevice, Device
        }, irqchip::*, tty::{
            serial::{ns16550a::Uart16550Driver, SerialDriver, UartDriver}, 
            tty_core::{CharDevice, TtyStruct}
        }, vf2::Vf2SDIO, VirtIoBlkDev, VirtIoHalImpl
    }, 
    hal::config::{DEVICE_ADDR_OFFSET, KERNEL_ADDR_OFFSET}
};


pub struct DeviceManager {
    pub char_devs: BTreeMap<(CharMajorNum, usize), Arc<dyn CharDevice>>,
    pub blk_devs: BTreeMap<(BlockMajorNum, usize), Arc<dyn BlockDevice>>,
    // pub drivers: Vec<Arc<dyn PhysDriver>>,
    pub irq_table: Vec<HardIrqHandler>,
    
    // pub UART_DRIVER: Option<Arc<dyn UartDriver>>
    pub FDT: Option<flat_device_tree::Fdt<'static>>,

    pub ICU: Option<Arc<dyn IrqController>>,

    // pub mods: Vec<dyn AbsDriverModule>,
    // 不是好的设计，摆烂
    pub uarts: Vec<Arc<dyn UartDriver>>,

    serials: Vec<Arc<SerialDriver>>,

    pub virtblks_mmio: Vec< Arc< VirtIoBlkDev<VirtIoHalImpl, MmioTransport<'static>> > >,
    pub virtblks_pci: Vec< Arc< VirtIoBlkDev<VirtIoHalImpl, PciTransport> > >,
    pub vf2_sdcards: Vec< Arc< Vf2SDIO >>

    // /// interrupt controller unit
    // pub ICU: Option<Arc<super::PLIC>>,
    // /// cpus
    // pub CPUS: Vec<Arc<super::CPU>>,
    // TODO: block drivers
    // pub blocks: Vec<>


}


impl DeviceManager {
    pub fn new() -> DeviceManager {
        DeviceManager {
            char_devs: BTreeMap::new(),
            blk_devs: BTreeMap::new(),
            // drivers: Vec::new(),
            irq_table: vec![HardIrqHandler::new(); 256],
            // UART_DRIVER: None,
            FDT: None,
            ICU: None,

            uarts: Vec::new(),
            serials: Vec::new(),

            virtblks_mmio: Vec::new(),
            virtblks_pci: Vec::new(),
            vf2_sdcards: Vec::new(),
        }
    }
    pub fn validate_raw_fdt(&mut self, root_addr: usize) {
        self.FDT = unsafe{ Fdt::from_ptr(root_addr as _) }.ok();
    }






    // pub fn probe_icu(&mut self) {
    //     #[cfg(target_arch = "riscv64")]
    //     {
    //         self.probe_plic();
    //     }
    //     #[cfg(target_arch = "loongarch64")]
    //     {
    //         #[cfg(feature = "board_qemu")]
    //         {
    //             self.probe_ls_eiointc();   
    //         }
    //         #[cfg(feature = "2k1000la")]
    //         {
    //             // self.probe_plic();
    //         }
    //     }
    // }



    pub fn probe_uarts(&mut self) {
        // TODO: more type of uart
        let uart0 = Uart16550Driver::probe(&self.FDT.unwrap());
        if let Some(uart0) = uart0 {
            self.uarts.push(uart0);
        }
    }

    fn register_serials(&mut self) {
        for uart in self.uarts.iter() {
            let irq_number = uart.irq_number();
            let serial = Arc::new(SerialDriver::new(uart.clone()));
            self.serials.push(serial.clone());
            if let Some(irq_number) = irq_number {
                if let Some(icu) = &self.ICU {
                    const MAX_CORES: usize = 1;
                    for core_id in 0..MAX_CORES {
                        icu.enable_irq(core_id, irq_number);
                    }
                }
                self.irq_table[irq_number].register(serial.clone());
            }
        }
    }

    pub fn register_ttys(&mut self) {
        self.register_serials();
        let major = CharMajorNum::Tty;
        let mut minor = 0;
        // TODO: virtual console
        minor = 64;
        for serial in self.serials.iter() {
            let tty = TtyStruct::new(serial.clone(), MajorNumber::Char(major), minor);
            self.char_devs.insert((major, minor),  Arc::new(tty));
            minor += 1;
        }
    }

    pub fn probe_virtio_blks(&mut self) {
        // from mmio
        let virtio_blk = VirtIoBlkDev::<VirtIoHalImpl, MmioTransport>::probe(&self.FDT.unwrap());
        if let Some(virtio_blk) = virtio_blk {
            self.virtblks_mmio.push(virtio_blk);
        }

        // from pci
        let virtio_blk = VirtIoBlkDev::<VirtIoHalImpl, PciTransport>::probe(&self.FDT.unwrap());
        if let Some(virtio_blk) = virtio_blk {
            self.virtblks_pci.push(virtio_blk);
        }

    }

    pub fn probe_vf2_sdcards(&mut self) {
        let vf2_sdcard = Vf2SDIO::probe(&self.FDT.unwrap());
        if let Some(vf2_sdcard) = vf2_sdcard {
            self.vf2_sdcards.push(vf2_sdcard);
        }
    }

    pub fn probe_and_register_ls2k_ahic(&mut self) {
        let ahci = AhciBlock::probe(&self.FDT.unwrap());
        let Some(ahci) = ahci else {
            panic!("[DeviceManager] ahci not found");
        };
        let MajorNumber::Block(major) = ahci.get_major() else {
            panic!("[DeviceManager] bad ahci");
        };
        let minor = ahci.get_minor();
        self.blk_devs.insert((major, minor), ahci);
    }

    pub fn register_virtio_blk_devs(&mut self) {
        /// 不重复就行
        // let mut minor = 0;
        for blk in self.virtblks_mmio.iter() {
            // let virt_blk_dev = Arc::new(VirtIoBlkDev::new(blk.clone(), MajorNumber::Block(major), minor));
            let virt_blk_dev = blk.clone();
            let MajorNumber::Block(major) = blk.get_major() else { continue; };
            let minor = blk.get_minor();
            let ret = self.blk_devs.insert((major, minor), virt_blk_dev);
            assert!(ret.is_none());
        }

        for blk in self.virtblks_pci.iter() {
            // let virt_blk_dev = Arc::new(VirtIoBlkDev::new(blk.clone(), MajorNumber::Block(major), minor));
            let virt_blk_dev = blk.clone();
            let MajorNumber::Block(major) = blk.get_major() else { continue; };
            let minor = blk.get_minor();
            let ret = self.blk_devs.insert((major, minor), virt_blk_dev);
            assert!(ret.is_none());
        }
    }

    pub fn register_vf2_sd_devs(&mut self) {
        for blk in self.vf2_sdcards.iter() {
            // let virt_blk_dev = Arc::new(VirtIoBlkDev::new(blk.clone(), MajorNumber::Block(major), minor));
            let virt_blk_dev = blk.clone();
            let MajorNumber::Block(major) = blk.get_major() else { continue; };
            let minor = blk.get_minor();
            let ret = self.blk_devs.insert((major, minor), virt_blk_dev);
            assert!(ret.is_none());
        }
    }


    pub fn probe_initial(&mut self, root_addr: usize) {
        self.validate_raw_fdt(root_addr);
        self.probe_icu();
        self.probe_uarts();
        self.register_ttys();
        #[cfg(feature = "board_qemu")]
        {
            self.probe_virtio_blks();
            self.register_virtio_blk_devs();            
        }
        #[cfg(feature = "vf2")]
        {
            self.probe_vf2_sdcards();
            self.register_vf2_sd_devs();            
        }
        #[cfg(feature = "2k1000la")]
        {
            self.probe_and_register_ls2k_ahic();
        }

    }

    // pub fn register_char_dev(&mut self, dev: Arc<dyn CharDevice>, major: CharMajorNum, minor: usize) {
    //     self.char_devs.insert((major, minor), dev);
    // }

    pub fn get_char_dev(&self, major: CharMajorNum, minor: usize) -> Option<Arc<dyn CharDevice>> {
        self.char_devs.get(&(major, minor)).map(Arc::clone)
    }

    // pub fn register_block_dev(&mut self, dev: Arc<dyn BlockDevice>, major: BlockMajorNum, minor: usize) {
    //     self.blk_devs.insert((major, minor), dev);
    // }

    pub fn get_block_dev(&self, major: BlockMajorNum, minor: usize) -> Option<Arc<dyn BlockDevice>> {
        self.blk_devs.get(&(major, minor)).map(Arc::clone)
    }

    pub fn get_device(&self, major: MajorNumber, minor: usize) -> Option<Arc<dyn Device>> {
        match major {
            MajorNumber::Char(major) => self.get_char_dev(major, minor).map( | a | a as Arc<dyn Device> ),
            MajorNumber::Block(major) => self.get_block_dev(major, minor).map( | a | a as Arc<dyn Device> ),
        }
    }


    pub fn handle_irq(&self, hart_id: usize) {
        let irq_number = self.ICU
            .as_ref()
            .expect("[DeviceManager::handle_irq] Bad icu")
            .claim_irq(hart_id)
            .expect("[DeviceManager::handle_irq] irq number not found");
        self.irq_table[irq_number].handle_irq();
        self.ICU.as_ref().unwrap().finish_irq(hart_id, irq_number);
    }

    // pub fn 
}

// TODO: remove the lock
lazy_static!{
    pub static ref DEVICE_MANAGER: RwLock<DeviceManager> = RwLock::new(DeviceManager::new());
}