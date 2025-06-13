use crate::hal::config::KERNEL_ADDR_OFFSET;

#[cfg(target_arch = "riscv64")]
pub const CLOCK_FREQ: usize = 1250_0000;

#[cfg(target_arch = "loongarch64")]
pub const CLOCK_FREQ: usize = 1_0000_0000;

#[cfg(target_arch = "riscv64")]
pub const MEMORY_END: usize = 0xc000_0000 + KERNEL_ADDR_OFFSET; // 将用户和内核空间放在一起，高地址为内核空间

#[cfg(target_arch = "loongarch64")]
pub const MEMORY_END: usize = 0xa800_0000 + KERNEL_ADDR_OFFSET; // 将用户和内核空间放在一起，高地址为内核空间

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
    (0x1000_2000, 0x00_1000),
    (0x1000_3000, 0x00_1000),
    (0x1000_4000, 0x00_1000),
    (0x1000_5000, 0x00_1000),
    (0x1000_6000, 0x00_1000),
    (0x1000_7000, 0x00_1000),
    (0x1000_8000, 0x00_1000),
];
