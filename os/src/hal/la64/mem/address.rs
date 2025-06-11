use crate::hal::{
    KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET, KERNEL_PG_ADDR_BASE, KERNEL_PG_VADDR_MASK,
    KERNEL_VADDR_MASK, KERNEL_VPN_MASK,
};
use crate::mm::{Direct, PageNum, Paged, PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

impl Paged for VirtAddr {
    /// 内核页表的虚拟地址 转换为对应的直接映射窗口的虚拟地址
    fn direct_va(&self) -> VirtAddr {
        ((self.0 & KERNEL_PG_VADDR_MASK) | KERNEL_ADDR_OFFSET).into()
    }
    /// 将内核页表的虚拟地址转化为物理地址
    fn paged_pa(&self) -> PhysAddr {
        (self.0 & KERNEL_PG_VADDR_MASK).into()
    }
}

impl Direct for VirtAddr {
    /// 内核直接映射窗口的虚拟地址 转化为对应的内核页表的虚拟地址
    fn paged_va(&self) -> VirtAddr {
        ((self.0 & KERNEL_VADDR_MASK) | KERNEL_PG_ADDR_BASE).into()
    }
    /// 内核直接映射窗口的虚拟地址转化为物理地址
    fn direct_pa(&self) -> PhysAddr {
        (self.0 & KERNEL_VADDR_MASK).into()
    }
}

impl Paged for PhysAddr {
    /// 物理地址转化为直接映射窗口的虚拟地址
    fn direct_va(&self) -> VirtAddr {
        (self.0 | KERNEL_ADDR_OFFSET).into()
    }
    fn paged_pa(&self) -> PhysAddr {
        self.clone()
    }
}

impl Direct for PhysAddr {
    /// 物理地址转化为内核页表的虚拟地址
    fn paged_va(&self) -> VirtAddr {
        (self.0 | KERNEL_PG_ADDR_BASE).into()
    }
    fn direct_pa(&self) -> PhysAddr {
        self.clone()
    }
}

impl PageNum for VirtPageNum {
    fn vpn(&self) -> VirtPageNum {
        self.clone()
    }
    fn ppn(&self) -> PhysPageNum {
        (self.0 & KERNEL_VPN_MASK).into()
    }
}

impl PageNum for PhysPageNum {
    fn vpn(&self) -> VirtPageNum {
        (self.0 | KERNEL_PGNUM_OFFSET).into()
    }
    fn ppn(&self) -> PhysPageNum {
        self.clone()
    }
}
