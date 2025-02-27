pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};

const KB: usize = 1024;
#[allow(unused)]
const MB: usize = 1024 * KB;

// TODO:目前是有栈协程，如果将userstack修改为8MB，kernelstack修改为64kB，会导致Kerenlstack爆栈
// 如果是无栈协程就不会
pub const USER_STACK_SIZE: usize = 8 * KB;
pub const KERNEL_STACK_SIZE: usize = 8 * KB;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;

pub const KERNEL_ADDR_OFFSET: usize = 0xffff_ffc0_0000_0000;

// When directly map: vpn = ppn + kernel direct offset
pub const KERNEL_PGNUM_OFFSET: usize = KERNEL_ADDR_OFFSET >> PAGE_SIZE_BITS;

pub const USER_SPACE_TOP: usize = 0x30_0000_0000;
pub const USER_TRAP_CONTEXT: usize = USER_SPACE_TOP - PAGE_SIZE;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

pub const HART_NUM: usize = 2;

/// hart start address
pub const HART_START_ADDR: usize = 0x80200000;

// File
/// max path name len
pub const PATH_MAX: usize = 4096;
/// 最大文件描述符数量
pub const RLIMIT_NOFILE: usize = 1024;
