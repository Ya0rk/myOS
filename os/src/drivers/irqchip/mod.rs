#[cfg(target_arch = "loongarch64")]
pub mod la64;

#[cfg(target_arch = "loongarch64")]
pub use la64::*;


#[cfg(target_arch = "riscv64")]
pub mod rv64;

#[cfg(target_arch = "riscv64")]
pub use rv64::*;




pub trait IrqController: Send + Sync {
    fn enable_irq(&self, hart_id: usize, irq_no: usize);
    fn disable_irq(&self, hart_id: usize, irq_no: usize);
    fn claim_irq(&self, hart_id: usize) -> Option<usize>;
    fn finish_irq(&self, hart_id: usize, irq_no: usize);
}