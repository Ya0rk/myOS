mod up;
mod interrupt;

pub use up::UPSafeCell;
pub use interrupt::{enable_interrupt, disable_interrupt, interrupt_is_enabled};