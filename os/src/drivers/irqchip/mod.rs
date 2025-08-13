pub mod loongson_pch_pic;
pub mod loongson_liointc;
pub mod loongson_eiointc;
pub mod riscv_plic;

pub use {
    loongson_pch_pic::PCHIntController,
    loongson_liointc::LocalIOIntController,
    loongson_eiointc::ExtIOIntController,
    riscv_plic::PLIC,
};


pub trait IrqController: Send + Sync {
    fn enable_irq(&self, hart_id: usize, irq_no: usize);
    fn disable_irq(&self, hart_id: usize, irq_no: usize);
    fn claim_irq(&self, hart_id: usize) -> Option<usize>;
    fn finish_irq(&self, hart_id: usize, irq_no: usize);
}