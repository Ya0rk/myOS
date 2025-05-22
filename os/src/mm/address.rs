//! Implementation of physical and virtual address and page number.
use core::ops::Range;

use sbi_spec::pmu::hardware_event::STALLED_CYCLES_FRONTEND;

// use super::PageTableEntry;
use crate::hal::mem::page_table::{PageTableEntry};
use crate::hal::config::{KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET, PAGE_MASK, PAGE_SIZE, PAGE_SIZE_BITS};
use crate::hal::KERNEL_PG_ADDR_BASE;
use core::fmt::{self, Debug, Formatter};
// use core::iter::Step;
/// physical address
/// TODO: move to hal/config
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;


macro_rules! impl_step {
    ($t:ty) => {
        impl core::iter::Step for $t {
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                usize::steps_between(&start.0, &end.0)
            }

            fn forward_checked(start: Self, count: usize) -> Option<Self> {
                usize::forward_checked(start.0, count).map(<$t>::from)
            }

            fn backward_checked(start: Self, count: usize) -> Option<Self> {
                usize::forward_checked(start.0, count).map(<$t>::from)
            }
        }
    };
}
macro_rules! impl_arithmetic_with_usize {
    ($t:ty) => {
        impl const core::ops::Add<usize> for $t {
            type Output = Self;
            #[inline]
            fn add(self, rhs: usize) -> Self {
                Self(self.0 + rhs)
            }
        }
        impl core::ops::AddAssign<usize> for $t {
            #[inline]
            fn add_assign(&mut self, rhs: usize) {
                *self = *self + rhs;
            }
        }
        impl core::ops::Sub<usize> for $t {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: usize) -> Self {
                Self(self.0 - rhs)
            }
        }
        impl core::ops::SubAssign<usize> for $t {
            #[inline]
            fn sub_assign(&mut self, rhs: usize) {
                *self = *self - rhs;
            }
        }
        impl core::ops::Sub<$t> for $t {
            type Output = usize;
            #[inline]
            fn sub(self, rhs: $t) -> usize {
                self.0 - rhs.0
            }
        }
    };
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KernelAddr(pub usize);
/// physical address
#[repr(C)]
#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
/// virtual address
#[repr(C)]
#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
/// physical page number
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
/// virtual page number
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl_step!(VirtPageNum);
impl_arithmetic_with_usize!(VirtPageNum);
impl_arithmetic_with_usize!(VirtAddr);

/// Debugging

impl Debug for KernelAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("KernelAddr:{:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

impl From<usize> for KernelAddr {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<KernelAddr> for PhysAddr {
    fn from(v: KernelAddr) -> Self {
        Self(v.0 - KERNEL_ADDR_OFFSET)
    }
}

impl From<KernelAddr> for PhysPageNum {
    fn from(v: KernelAddr) -> Self {
        PhysAddr::from(v).floor()
    }
}

impl From<PhysAddr> for KernelAddr {
    fn from(v: PhysAddr) -> Self {
        Self(v.0 + KERNEL_ADDR_OFFSET)
    }
}

impl From<KernelAddr> for VirtAddr {
    fn from(v: KernelAddr) -> Self {
        Self(v.0)
    }
}

impl From<VirtAddr> for KernelAddr {
    fn from(v: VirtAddr) -> Self {
        Self(v.0)
    }
}
/// todo 需要在loongarch中重建
/// 
fn check_addr_valid(addr: usize, offset: usize) {
    let tmp: isize = (addr as isize >> offset) as isize;
    assert!(tmp == 0 || tmp == -1, "invalid addr: {:#x}", addr);
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        // check_addr_valid(v, PA_WIDTH_SV39);
        Self(v)
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        // check_addr_valid(v, PPN_WIDTH_SV39);
        Self(v)
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        // 拓展虚拟地址到512GB，在这之前需要做检查
        // check_addr_valid(v, VA_WIDTH_SV39);
        Self(v)
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        let tmp = v >> (VPN_WIDTH_SV39 - 1);
        // let is_valid = tmp == 0 || tmp == (1 << (52 - VPN_WIDTH_SV39 + 1)) - 1;
        // assert!(is_valid, "invalid v to VirtPageNum: {:#x}", v);
        Self(v)
    }
}
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}
///
impl VirtAddr {
    ///`VirtAddr`->`VirtPageNum`
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    ///`VirtAddr`->`VirtPageNum`
    pub fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    ///Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_MASK
    }
    pub fn align_down(&self) -> Self {
        Self(self.0 & (!PAGE_MASK))
    }
    ///Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.0 as *mut u8
    }
    pub fn from_usize_range(range: core::ops::Range<usize>) -> core::ops::Range<Self> {
        Self(range.start)..Self(range.end)
    }
    pub fn to_usize(&self) -> usize {
        self.0
    }

}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        // assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    ///`PhysAddr`->`PhysPageNum`
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    ///`PhysAddr`->`PhysPageNum`
    pub fn ceil(&self) -> PhysPageNum {
        if self.0 == 0 {
            PhysPageNum(0)
        } else {
            PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    ///Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    ///Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

}
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        // assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    ///返回虚拟地址的三级索引
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

    pub fn to_vaddr(&self) -> VirtAddr {
        VirtAddr::from(self.0 << PAGE_SIZE_BITS)
    }
}

impl PhysAddr {
    ///Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
}
// TODO: replace it
impl KernelAddr {
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl PhysPageNum {
    pub fn to_paddr(&self) -> PhysAddr {
        (self.0 << PAGE_SIZE_BITS).into()
    }
    /// 取出当前节点的页表项数组
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        // TODO: new ptv
        let va = KernelAddr::from(pa).0;
        unsafe { core::slice::from_raw_parts_mut(va as *mut PageTableEntry, 512) }
    }
    /// 返回一个字节数组的可变引用
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        // TODO: new ptv
        let va = KernelAddr::from(pa).0;
        unsafe { core::slice::from_raw_parts_mut(va as *mut u8, 4096) }
    }
    ///
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        // TODO: new ptv
        let va = KernelAddr::from(pa);
        va.get_mut()
    }
    ///Get u8 array on `PhysPageNum` with given length
    pub fn get_bytes_array_from_offset(&self, offset: usize, len: usize) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        // TODO: new ptv
        let kernel_va = KernelAddr::from(pa).0 + offset;
        unsafe { core::slice::from_raw_parts_mut(kernel_va as *mut u8, len) }
    }
    pub fn get_bytes_array_from_range(&self, range: Range<usize>) -> &'static mut [u8] {
        debug_assert!(range.end <= PAGE_SIZE, "range: {range:?}");
        // TODO: new ptv
        let mut vaddr: VirtAddr = (self.to_paddr().0 + KERNEL_ADDR_OFFSET).into();
        vaddr += range.start;
        unsafe { core::slice::from_raw_parts_mut(vaddr.to_usize() as *mut u8, range.len()) }

    }
}

pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}
impl StepByOne for PhysPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
/// a simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;

// /// translate kernel virtual addr into physical addr
// pub fn kaddr_v2p(va: VirtAddr) -> PhysAddr {
//     (va.0 - KERNEL_ADDR_OFFSET).into()
// }


// // TODO: should not use on la
// /// translate kernel virtual page number into physical page number
// pub fn kpn_v2p(vpn: VirtPageNum) -> PhysPageNum {
//     (vpn.0 - KERNEL_PGNUM_OFFSET).into()
// }

// /// translate physical addr into kernel virtual addr
// pub fn kaddr_p2v(pa: PhysAddr) -> VirtAddr {
//     (pa.0 + KERNEL_ADDR_OFFSET).into()
// }


// // TODO: should not use on la
// /// translate physical page number into kernel virtual page number
// pub fn kpn_p2v(ppn: PhysPageNum) -> VirtPageNum {
//     (ppn.0 + KERNEL_PGNUM_OFFSET).into()
// }

// pub fn kva_pg2d(va: VirtAddr) -> VirtAddr {
//     (va.0 - KERNEL_PG_ADDR_BASE + KERNEL_ADDR_OFFSET).into()
// }

// pub fn kva_d2pg(va: VirtAddr) -> VirtAddr {
//     (va.0 - KERNEL_ADDR_OFFSET + KERNEL_PG_ADDR_BASE).into()
// }

pub trait Direct {
    fn direct_pa(&self) -> PhysAddr;
    fn paged_va(&self) -> VirtAddr;
}

pub trait Paged {
    fn paged_pa(&self) -> PhysAddr;
    fn direct_va(&self) -> VirtAddr;
}

pub trait PageNum {
    fn ppn(&self) -> PhysPageNum;
    fn vpn(&self) -> VirtPageNum;
}