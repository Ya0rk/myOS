use core::arch::asm;

/*
// for C ffi test
unsafe extern "C" {
    pub fn ahci_mdelay(ms: u32);
    pub fn ahci_printf(fmt: *const u8, _: ...) -> i32;
    pub fn ahci_malloc_align(size: u64, align: u32) -> u64;
    pub fn ahci_sync_dcache();
    pub fn ahci_phys_to_uncached(va: u64) -> u64;
    pub fn ahci_virt_to_phys(va: u64) -> u64;
}
*/

// 这里是测试时用于调用C的printf
// 替换成OS实现的printf
unsafe extern "C" {
    pub fn ahci_printf(fmt: *const u8, _: ...) -> i32;
}

// 等待数毫秒
pub fn ahci_mdelay(ms: u32) {}

// 同步dcache中所有cached和uncached访存请求
pub fn ahci_sync_dcache() {
    unsafe {
        asm!("dbar 0");
    }
}

// 分配按align字节对齐的内存
pub fn ahci_malloc_align(size: u64, align: u32) -> u64 {
    0
}

// 物理地址转换为uncached虚拟地址
pub fn ahci_phys_to_uncached(pa: u64) -> u64 {
    pa
}

// cached虚拟地址转换为物理地址
// ahci dma可以接受64位的物理地址
pub fn ahci_virt_to_phys(va: u64) -> u64 {
    va
}
