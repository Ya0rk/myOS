mod blk;
pub mod probe;
mod pci;

pub use blk::*;
use log::info;
use lwext4_rust::bindings::printf;
use spin::Mutex;
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};
use core::ptr::NonNull;

use crate::{
    drivers::DevError, mm::{
        frame_alloc, frame_dealloc, FrameTracker, KernelAddr, PageTable, PhysPageNum,
        StepByOne, VirtAddr,
    }, sync::SpinNoIrqLock, task::current_user_token
};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
lazy_static! {
    /// 实现 Trait BlockDevice时对内部操作加锁
    // pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
    static ref QUEUE_FRAMES: SpinNoIrqLock<Vec<Arc<FrameTracker>>> = SpinNoIrqLock::new(Vec::new());
}

pub struct VirtIoHalImpl;

unsafe impl Hal for VirtIoHalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let mut ppn_base = PhysPageNum(0);
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.lock().push(Arc::new(frame));
        }
        let pa: crate::mm::address::PhysAddr = ppn_base.into();
        let va = KernelAddr::from(pa).0;
        let vaddr = if let Some(vaddr) = NonNull::new(va as _) {
            vaddr
        } else {
            panic!("dma alloc error");
        };
        (pa.0, vaddr)
    }

    unsafe fn dma_dealloc(pa: PhysAddr, va: NonNull<u8>,pages: usize) -> i32 {
        let pa = crate::mm::address::PhysAddr::from(pa);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    unsafe fn mmio_phys_to_virt(pa: PhysAddr, _size: usize) -> NonNull<u8> {
        let va = pa + KERNEL_ADDR_OFFSET;
        // println!("[mmio_phys_to_virt] {:#x}", va);
        NonNull::new(va as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // println!("[share] {:#x}", vaddr);
        // info!("[share] buffer vaddr is {:#x}, user token is {:#x}, kernel token is {:#x}", vaddr, current_user_token(), current_kernel_token());
        // match vaddr >> 60 {
        //     0x9 | 0x8 => {
        //         vaddr & 0x0fff_ffff_ffff_ffff
        //     },
        //     0xF => {
        //         PageTable::from_token(current_kernel_token())
        //         .translate_va(VirtAddr::from(vaddr))
        //         .unwrap()
        //         .0
        //     },
        //     0x0 => {
        //         PageTable::from_token(current_user_token())
        //         .translate_va(VirtAddr::from(vaddr))
        //         .unwrap()
        //         .0
        //     },
        //     _ => {
        //         panic!("Invalid Virtual Address");
        //     }
        // }
        // Nothing to do, as the host already has access to all memory.
        
        vaddr - KERNEL_ADDR_OFFSET
        // 注意到现在采取直接映射模式,在entry中有设置
        // vaddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }

    // // 这些属性中的接口已经被删除
    // fn phys_to_virt(addr: usize) -> usize {
    //     KernelAddr::from(PhysAddr::from(addr)).0
    // }

    // fn virt_to_phys(vaddr: usize) -> usize {
    //     // info!("kkkkkk");
    //     PageTable::from_token(current_user_token())
    //         .translate_va(VirtAddr::from(vaddr))
    //         .unwrap()
    //         .0
    //     // PhysAddr::from(vaddr - KERNEL_ADDR_OFFSET).0
    // }
}

#[allow(dead_code)]
const fn as_dev_err(e: virtio_drivers::Error) -> DevError {
    use virtio_drivers::Error::*;
    match e {
        NotReady => DevError::Again,
        AlreadyUsed => DevError::AlreadyExists,
        InvalidParam => DevError::InvalidParam,
        DmaError => DevError::NoMemory,
        IoError => DevError::Io,
        _ => DevError::BadState,
    }
}
