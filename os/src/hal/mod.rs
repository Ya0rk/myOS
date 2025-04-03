//! `os/src/hal/mod.rs`
//! 硬件抽象层Hardware Abstraction Layer


#[cfg(target_arch = "riscv64")]
pub mod rv64;
#[cfg(target_arch = "riscv64")]
pub use rv64::*;

#[cfg(target_arch = "loongarch64")]
pub mod la64;
#[cfg(target_arch = "loongarch64")]
pub use la64::*;





