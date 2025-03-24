#[cfg(target_arch = "riscv64")]
mod rv64;
#[cfg(target_arch = "riscv64")]
pub use self::rv64::*;


#[cfg(target_arch = "loongarch64")]
mod la64;
#[cfg(target_arch = "loongarch64")]
pub use self::la64::*;



