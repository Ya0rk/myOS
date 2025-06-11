//! `os/src/hal/la64`
//! Loongarch架构相关的mod

pub mod arch;
pub mod config;
pub mod entry;
pub mod mem;
pub mod trap;

pub use config::*;
