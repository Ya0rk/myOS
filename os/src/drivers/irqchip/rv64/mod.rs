pub mod riscv_plic;
use alloc::sync::Arc;
pub use riscv_plic::PLIC;

use crate::drivers::{device_new::{dev_core::PhysDriverProbe as _, manager::DeviceManager}, irqchip::IrqController};

impl DeviceManager {
    pub fn probe_riscv_plic(&mut self) {
        let icu = PLIC::probe(&self.FDT.unwrap());
        self.ICU = icu.map(| icu | icu as Arc<dyn IrqController>) ;
    }
    pub fn probe_icu(&mut self) {

        self.probe_riscv_plic();

    }

}

