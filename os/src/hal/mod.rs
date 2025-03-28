// os/src/hal/mod.rs

// #[cfg(target_arch = "loongarch64")]
pub use arch::la64::timer;
// #[cfg(target_arch = "loongarch64")]
pub use arch::la64::interrupt;

// #[cfg(target_arch = "riscv64")]
// pub use arch::rv64::timer;
// #[cfg(target_arch = "riscv64")]
// pub use arch::rv64::interrupt;

pub mod arch;
