pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};

const KB: usize = 1024;
#[allow(unused)]
const MB: usize = 1024 * KB;

// mm
// TODO:目前是有栈协程，如果将userstack修改为8MB，kernelstack修改为64kB，会导致Kerenlstack爆栈
// 如果是无栈协程就不会
pub const USER_STACK_SIZE: usize = 8 * KB;
pub const KERNEL_STACK_SIZE: usize = 8 * KB;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;

pub const PAGE_MASK: usize = PAGE_SIZE - 1;

pub const PTE_SIZE: usize = 8;
pub const PTES_PER_PAGE: usize = PAGE_SIZE / PTE_SIZE;
/// 3 level for sv39 page table
pub const PAGE_TABLE_LEVEL_NUM: usize = 3;

pub const KERNEL_ADDR_OFFSET: usize = 0xffff_ffc0_0000_0000;
// When directly map: vpn = ppn + kernel direct offset
pub const KERNEL_PGNUM_OFFSET: usize = KERNEL_ADDR_OFFSET >> PAGE_SIZE_BITS;
pub const USER_SPACE_TOP: usize = 0x30_0000_0000;
pub const USER_TRAP_CONTEXT: usize = USER_SPACE_TOP - PAGE_SIZE;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
/// hart start address
pub const HART_START_ADDR: usize = 0x80200000;

// scheudler
pub const HART_NUM: usize = 2;
#[allow(unused)]
pub const IDLE_PID: usize = 0;
pub const INITPROC_PID: usize = 1;


// File system
/// max path name len
pub const PATH_MAX: usize = 4096;
/// 最大文件描述符数量
pub const RLIMIT_NOFILE: usize = 1024;
/// 当 filename 为相对路径的情况下将当前进程的工作目录设置为起始路径
pub const AT_FDCWD: isize = -100;


/// From Phoenix
pub const BLOCK_SIZE: usize = 512;
pub const MMAP_PRE_ALLOC_PAGES: usize = 8;
// pub const USER_STACK_SIZE: usize = 8 * 1024 * 1024;
pub const USER_STACK_PRE_ALLOC_SIZE: usize = 4 * PAGE_SIZE;


pub const USER_ELF_PRE_ALLOC_PAGE_CNT: usize = 0;

/// Dynamic linked interpreter address range in user space
pub const DL_INTERP_OFFSET: usize = 0x20_0000_0000;

pub const MAX_BUFFER_HEADS: usize = 0x18000;
pub const MAX_BUFFER_CACHE: usize = 0x1000;
pub const MAX_BUFFER_PAGES: usize = MAX_BUFFER_CACHE / MAX_BUFFERS_PER_PAGE;
pub const MAX_BUFFERS_PER_PAGE: usize = PAGE_SIZE / BLOCK_SIZE;
pub const BUFFER_NEED_CACHE_CNT: usize = 8;

/// User stack segment
pub const U_SEG_STACK_BEG: usize = 0x0000_0001_0000_0000;
pub const U_SEG_STACK_END: usize = 0x0000_0002_0000_0000;

/// User heap segment
// pub const U_SEG_HEAP_BEG: usize = 0x0000_0002_0000_0000;
// pub const U_SEG_HEAP_END: usize = 0x0000_0004_0000_0000;
pub const U_SEG_HEAP_BEG: usize = 0x0000_0000_4000_0000;
pub const U_SEG_HEAP_END: usize = 0x0000_0000_8000_0000;

/// User mmap segment
pub const U_SEG_FILE_BEG: usize = 0x0000_0004_0000_0000;
pub const U_SEG_FILE_END: usize = 0x0000_0006_0000_0000;

/// User share segment
pub const U_SEG_SHARE_BEG: usize = 0x0000_0006_0000_0000;
pub const U_SEG_SHARE_END: usize = 0x0000_0008_0000_0000;

// =========== Kernel segments ===========
pub const K_SEG_BEG: usize = 0xffff_ffc0_0000_0000;

// Kernel heap segment (64 GiB)
pub const K_SEG_HEAP_BEG: usize = 0xffff_ffc0_0000_0000;
pub const K_SEG_HEAP_END: usize = 0xffff_ffd0_0000_0000;

// File mapping (64 GiB)
pub const K_SEG_FILE_BEG: usize = 0xffff_ffd0_0000_0000;
pub const K_SEG_FILE_END: usize = 0xffff_ffe0_0000_0000;

// Physical memory mapping segment (62 GiB)
pub const K_SEG_PHY_MEM_BEG: usize = 0xffff_fff0_0000_0000;
pub const K_SEG_PHY_MEM_END: usize = 0xffff_ffff_8000_0000;

// Kernel text segment (1 GiB)
pub const K_SEG_TEXT_BEG: usize = 0xffff_ffff_8000_0000;
pub const K_SEG_TEXT_END: usize = 0xffff_ffff_c000_0000;

// DTB fixed mapping
pub const K_SEG_DTB_BEG: usize = K_SEG_DTB_END - MAX_DTB_SIZE;
pub const K_SEG_DTB_END: usize = 0xffff_ffff_f000_0000;
pub const MAX_DTB_SIZE: usize = PAGE_SIZE * PAGE_SIZE;

/// End From Phoenix


pub fn align_down_by_page(addr: usize) -> usize {
    addr & !PAGE_MASK
}

pub fn align_up_by_page(addr: usize) -> usize {
    (addr + PAGE_MASK) & !PAGE_MASK
}

pub fn is_aligned_to_page(addr: usize) -> bool {
    (addr & PAGE_MASK) == 0
}
pub const PIPE_BUFFER_SIZE: usize = 0x10000;