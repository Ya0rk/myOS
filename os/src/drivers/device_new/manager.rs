use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::{Vec}};
#[macro_use]
use alloc::vec;
use embedded_hal::serial;
use flat_device_tree::Fdt;
use spin::{rwlock::RwLock};

use crate::drivers::{device_new::{dev_core::{PhysDriver, PhysDriverProbe}, dev_number::MajorNumber, irq::{HandleHardIrq, HardIrqHandler}, Device}, tty::{serial::{ns16550a::Uart16550Driver, SerialDriver, UartDriver}, tty_core::TtyStruct}};


pub struct DeviceManager {
    pub device_table: BTreeMap<(MajorNumber, usize), Arc<dyn Device>>,
    // pub drivers: Vec<Arc<dyn PhysDriver>>,
    pub irq_table: Vec<HardIrqHandler>,
    
    // pub UART_DRIVER: Option<Arc<dyn UartDriver>>
    pub FDT: Option<flat_device_tree::Fdt<'static>>,

    // pub mods: Vec<dyn AbsDriverModule>,
    pub uarts: Vec<Arc<dyn UartDriver>>,

    serials: Vec<Arc<SerialDriver>>,

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
            device_table: BTreeMap::new(),
            // drivers: Vec::new(),
            irq_table: vec![HardIrqHandler::new(); 256],
            // UART_DRIVER: None,
            FDT: None,

            uarts: Vec::new(),
            serials: Vec::new(),
        }
    }
    pub fn validate_raw_fdt(&mut self, root_addr: usize) {
        self.FDT = unsafe{ Fdt::from_ptr(root_addr as _) }.ok();
    }

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
                self.irq_table[irq_number].register(serial.clone());
            }
        }
    }

    pub fn register_ttys(&mut self) {
        self.register_serials();
        let major = MajorNumber::Tty;
        let mut minor = 0;
        // TODO: virtual console
        minor = 64;
        for serial in self.serials.iter() {
            let tty = TtyStruct::new(serial.clone(), major, minor);
            self.device_table.insert((major, minor),  Arc::new(tty));
            minor += 1;
        }
    }
    pub fn probe_initial(&mut self, root_addr: usize) {
        self.validate_raw_fdt(root_addr);
        self.probe_uarts();
        self.register_ttys();
    }

    pub fn get_device(&self, major: MajorNumber, minor: usize) -> Option<Arc<dyn Device>> {
        self.device_table.get(&(major, minor)).map(Arc::clone)
    }

    pub fn handle_irq(&self, irq_number: usize) {
        self.irq_table[irq_number].handle_irq();
    }
}

lazy_static!{
    pub static ref DEVICE_MANAGER: RwLock<DeviceManager> = RwLock::new(DeviceManager::new());
}