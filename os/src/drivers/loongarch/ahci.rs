use crate::hal::config::PAGE_SIZE;
use crate::mm::{frame_alloc, frame_dealloc, PhysAddr};

const BLOCK_SZ: usize = crate::hal::config::BLOCK_SIZE;

use isomorphic_drivers::{
    block::ahci::{AHCI, BLOCK_SIZE},
    provider,
};
use log::info;
use spin::mutex::Mutex;

use super::pci::pci_init;

pub struct AHCIDriver(Mutex<AHCI<Provider>>);

impl AHCIDriver {
    pub fn new() -> Self {
        Self(Mutex::new(pci_init().expect("AHCI new failed")))
    }
}

impl AHCIDriver {
    fn read_block(&self, mut block_id: usize, buf: &mut [u8]) {
        // kernel BLOCK_SZ=2048, SATA BLOCK_SIZE=512，four times
        block_id = block_id * (BLOCK_SZ / BLOCK_SIZE);
        for buf in buf.chunks_mut(BLOCK_SIZE) {
            self.0
                .lock()
                .read_block(block_id as usize, buf);
            block_id += 1;
        }
    }

    fn write_block(&self, mut block_id: usize, buf: &[u8]) {
        block_id = block_id * (BLOCK_SZ / BLOCK_SIZE);
        for buf in buf.chunks(BLOCK_SIZE) {
            self.0
                .lock()
                .write_block(block_id as usize, buf);
            block_id += 1;
        }
    }
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = AHCIDriver::new();
    let mut write_buffer = [0u8; BLOCK_SZ];
    let mut read_buffer = [0u8; BLOCK_SZ];
    for i in 0..BLOCK_SZ {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}


pub struct Provider;

impl provider::Provider for Provider {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_dma(size: usize) -> (usize, usize) {
        let pages = size / PAGE_SIZE;
        let mut base = 0;
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            let frame_pa: PhysAddr = frame.ppn.into();
            let frame_pa = frame_pa.into();
            core::mem::forget(frame);
            if i == 0 {
                base = frame_pa;
            }
            assert_eq!(frame_pa, base + i * PAGE_SIZE);
        }
        let base_page = base / PAGE_SIZE;
        info!("virtio_dma_alloc: {:#x} {}", base_page, pages);
        (base, base)
    }

    fn dealloc_dma(va: usize, size: usize) {
        info!("dealloc_dma: {:x} {:x}", va, size);
        let pages = size / PAGE_SIZE;
        let mut pa = va;
        for _ in 0..pages {
            frame_dealloc(PhysAddr::from(pa).into());
            pa += PAGE_SIZE;
        }
    }
}
