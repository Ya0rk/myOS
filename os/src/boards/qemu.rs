//! Constants used in Core for qemu

use crate::config::KERNEL_ADDR_OFFSET;

pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000 + KERNEL_ADDR_OFFSET; // 将用户和内核空间放在一起，高地址为内核空间

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];
