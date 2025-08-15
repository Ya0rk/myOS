use crate::hal::config::KERNEL_ADDR_OFFSET;

#[cfg(all(target_arch = "riscv64", not(feature = "vf2")))]
pub const CLOCK_FREQ: usize = 1250_0000;

#[cfg(target_arch = "loongarch64")]
pub const CLOCK_FREQ: usize = 1_0000_0000;

#[cfg(all(target_arch = "riscv64", feature = "vf2"))]
pub const CLOCK_FREQ: usize = 10_000_000; // 这里的频率可能有误， 400_0000

#[cfg(feature = "board_qemu")]
pub const MEMORY_END: usize = 0xc000_0000 + KERNEL_ADDR_OFFSET; // 将用户和内核空间放在一起，高地址为内核空间

#[cfg(feature = "vf2")]
pub const MEMORY_END: usize = 0xc000_0000 + KERNEL_ADDR_OFFSET; // 将用户和内核空间放在一起，高地址为内核空间

#[cfg(feature = "2k1000la")]
pub const MEMORY_END: usize = 0x1_0000_0000 + KERNEL_ADDR_OFFSET;

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
