pub mod boot;

core::arch::global_asm!(include_str!("entry.asm"));