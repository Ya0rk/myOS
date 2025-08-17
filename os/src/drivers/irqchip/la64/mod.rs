pub mod loongson_eiointc;
pub mod loongson_liointc;
pub mod loongson_pch_pic;

use {crate::{drivers::{device::manager::DeviceManager, irqchip::IrqController as _}, hal::DEVICE_ADDR_OFFSET}, alloc::sync::Arc};
pub use {
    loongson_pch_pic::PCHIntController,
    loongson_liointc::LocalIOIntController,
    loongson_eiointc::ExtIOIntController,
};

impl DeviceManager {
    pub fn probe_ls_eiointc(&mut self) {
        let pic = unsafe { PCHIntController::new(0x10000000 + DEVICE_ADDR_OFFSET) };
        let icu = ExtIOIntController::new(0x1fe00000 + DEVICE_ADDR_OFFSET, Arc::new(pic));
        icu.device_enable();
        icu.enable_irq(0, 1);
        icu.debug_send(1);
        self.ICU = Some(Arc::new(icu));
    }

    pub fn probe_icu(&mut self) {

        #[cfg(feature = "board_qemu")]
        {
            self.probe_ls_eiointc();   
        }
        #[cfg(feature = "2k1000la")]
        {
            // self.probe_plic();
        }

    }
}