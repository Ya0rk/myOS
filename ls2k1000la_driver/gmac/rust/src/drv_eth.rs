#![allow(dead_code, unused_assignments, unused_mut)]

use crate::eth_defs::*;
use crate::eth_dev::*;
use crate::platform::*;

// 检查rgmii链路状态
// eth_update_linkstate通知操作系统链路状态
pub fn eth_phy_rgsmii_check(gmacdev: &mut net_device) {
    let mut value: u32 = 0;
    let mut status: u32 = 0;

    value = eth_mac_read_reg(gmacdev.MacBase, GmacRgsmiiStatus);
    status = value & (MacLinkStatus >> MacLinkStatusOff);

    if gmacdev.LinkStatus != status {
        unsafe { eth_update_linkstate(gmacdev, status) };
    }

    if status != 0 {
        gmacdev.LinkStatus = 1;
        gmacdev.DuplexMode = value & MacLinkMode;
        let mut speed: u32 = value & MacLinkSpeed;
        if speed == MacLinkSpeed_125 {
            gmacdev.Speed = 1000;
        } else if speed == MacLinkSpeed_25 {
            gmacdev.Speed = 100;
        } else {
            gmacdev.Speed = 10;
        }
        unsafe {
            eth_printf(
                b"Link is Up - %u Mpbs / %s Duplex\n\0" as *const u8,
                gmacdev.Speed,
                if gmacdev.DuplexMode != 0 {
                    b"Full\0" as *const u8
                } else {
                    b"Half\0" as *const u8
                },
            )
        };
    } else {
        gmacdev.LinkStatus = 0;
        unsafe { eth_printf(b"Link is Down\n\0" as *const u8) };
    };
}

// 初始化phy
pub fn eth_phy_init(gmacdev: &mut net_device) {
    let mut phy: u32 = 0;
    let mut data: u32 = 0;

    data = eth_mdio_read(gmacdev.MacBase, gmacdev.PhyBase as u32, 2) as u32;
    phy |= data << 16;
    data = eth_mdio_read(gmacdev.MacBase, gmacdev.PhyBase as u32, 3) as u32;
    phy |= data;

    match phy {
        0x0000010a => {
            unsafe {
                eth_printf(
                    b"probed ethernet phy YT8511H/C, id 0x%08x\n\0" as *const u8,
                    phy,
                )
            };
        }
        _ => {
            unsafe {
                eth_printf(
                    b"probed unknown ethernet phy, id 0x%08x\n\0" as *const u8,
                    phy,
                )
            };
        }
    };
}

pub fn eth_handle_tx_over(gmacdev: &mut net_device) {
    loop {
        let mut desc_idx: u32 = gmacdev.TxBusy;
        let mut txdesc: DmaDesc = unsafe { gmacdev.TxDesc[desc_idx as usize].read() } as DmaDesc;

        if eth_get_desc_owner(&txdesc) || eth_is_desc_empty(&txdesc) {
            break;
        }

        if eth_is_tx_desc_valid(&txdesc) {
            let mut length: u32 = (txdesc.length & DescSize1Mask) >> DescSize1Shift;
            gmacdev.tx_bytes += length as u64;
            gmacdev.tx_packets += 1;
        } else {
            gmacdev.tx_errors += 1;
        }

        let is_last: bool = eth_is_last_tx_desc(&txdesc);
        txdesc.status = if is_last { TxDescEndOfRing } else { 0 };
        txdesc.length = 0;
        txdesc.buffer1 = 0;
        txdesc.buffer2 = 0;
        unsafe {
            gmacdev.TxDesc[desc_idx as usize].write(txdesc);
        }

        gmacdev.TxBusy = if is_last { 0 } else { desc_idx + 1 };
    }
}

// 操作系统传递接收数据的单元pbuf给驱动
// pbuf可能是操作系统自定义结构
// 返回接收到的数据字节数
#[unsafe(no_mangle)]
pub extern "C" fn eth_tx(gmacdev: &mut net_device, pbuf: u64) -> i32 {
    let mut buffer: u64 = 0;
    let mut length: u32 = 0;
    let mut dma_addr: u32 = 0;
    let mut desc_idx: u32 = gmacdev.TxNext;
    let mut txdesc: DmaDesc = unsafe { gmacdev.TxDesc[desc_idx as usize].read() } as DmaDesc;
    let mut is_last: bool = eth_is_last_tx_desc(&txdesc);

    if eth_get_desc_owner(&txdesc) {
        return -1;
    }

    buffer = gmacdev.TxBuffer[desc_idx as usize];
    length = unsafe { eth_handle_tx_buffer(pbuf, buffer) };
    dma_addr = unsafe { eth_virt_to_phys(buffer) };

    txdesc.status |= DescOwnByDma | DescTxIntEnable | DescTxLast | DescTxFirst;
    txdesc.length = length << DescSize1Shift & DescSize1Mask;
    txdesc.buffer1 = dma_addr;
    txdesc.buffer2 = 0;
    unsafe {
        gmacdev.TxDesc[desc_idx as usize].write(txdesc);
    }

    gmacdev.TxNext = if is_last { 0 } else { desc_idx + 1 };

    unsafe { eth_sync_dcache() };

    eth_gmac_resume_dma_tx(gmacdev);

    return 0;
}

// pbuf是返回给操作系统的数据单元
// 可能是操作系统自定义结构
#[unsafe(no_mangle)]
pub extern "C" fn eth_rx(gmacdev: &mut net_device) -> u64 {
    let mut desc_idx: u32 = gmacdev.RxBusy;
    let mut rxdesc: DmaDesc = unsafe { gmacdev.RxDesc[desc_idx as usize].read() } as DmaDesc;
    let mut is_last: bool = eth_is_last_rx_desc(&rxdesc);

    if eth_is_desc_empty(&rxdesc) || eth_get_desc_owner(&rxdesc) {
        eth_dma_enable_interrupt(gmacdev, DmaIntEnable);
        return 0;
    }

    let mut pbuf: u64 = 0;
    let mut dma_addr = rxdesc.buffer1;

    if eth_is_rx_desc_valid(&rxdesc) {
        let mut length: u32 = eth_get_rx_length(&rxdesc);
        let mut buffer: u64 = unsafe { eth_phys_to_virt(dma_addr) };

        unsafe {
            eth_sync_dcache();
        }

        pbuf = unsafe { eth_handle_rx_buffer(buffer, length) };
        gmacdev.rx_bytes += length as u64;
        gmacdev.rx_packets += 1;
    } else {
        gmacdev.rx_errors += 1;
    }

    rxdesc.status = DescOwnByDma;
    rxdesc.length = if is_last { RxDescEndOfRing } else { 0 };
    rxdesc.length |= (2048) << DescSize1Shift & DescSize1Mask;
    rxdesc.buffer1 = dma_addr;
    rxdesc.buffer2 = 0;
    unsafe {
        gmacdev.RxDesc[desc_idx as usize].write(rxdesc);
    }

    gmacdev.RxBusy = if is_last { 0 } else { desc_idx + 1 };
    return pbuf;
}

// 中断处理程序
// eth_rx_ready通知操作系统可以接收数据
// eth_handle_tx_over用于处理已经发送完的描述符
#[unsafe(no_mangle)]
pub extern "C" fn eth_irq(gmacdev: &mut net_device) {
    let mut dma_status: u32 = 0;
    let mut dma_int_enable: u32 = DmaIntEnable;

    dma_status = eth_mac_read_reg(gmacdev.DmaBase, DmaStatus);
    if dma_status == 0 {
        return;
    }

    eth_dma_disable_interrupt_all(gmacdev);

    if dma_status & GmacPmtIntr != 0 {
        unsafe { eth_printf(b"gmac pmt interrupt\n\0" as *const u8) };
    }
    if dma_status & GmacMmcIntr != 0 {
        unsafe { eth_printf(b"gmac mmc interrupt\n\0" as *const u8) };
    }
    if dma_status & GmacLineIntfIntr != 0 {
        eth_mac_read_reg(gmacdev.MacBase, GmacInterruptStatus);
        eth_mac_read_reg(gmacdev.MacBase, GmacInterruptMask);
        if eth_mac_read_reg(gmacdev.MacBase, GmacInterruptStatus) & GmacRgmiiIntSts != 0 {
            eth_mac_read_reg(gmacdev.MacBase, GmacRgsmiiStatus);
        }
        eth_phy_rgsmii_check(gmacdev);
    }

    eth_mac_write_reg(gmacdev.DmaBase, DmaStatus, dma_status);

    if dma_status & DmaIntBusError != 0 {
        unsafe { eth_printf(b"gmac fatal bus error interrupt\n\0" as *const u8) };
    }
    if dma_status & DmaIntRxStopped != 0 {
        unsafe { eth_printf(b"gmac receive process stopped\n\0" as *const u8) };
        eth_dma_enable_rx(gmacdev);
    }
    if dma_status & DmaIntRxNoBuffer != 0 {
        unsafe { eth_printf(b"gmac receive buffer unavailable\n\0" as *const u8) };
        dma_int_enable &= !DmaIntRxNoBuffer;
        eth_gmac_resume_dma_rx(gmacdev);
        unsafe { eth_rx_ready(gmacdev) };
    }
    if dma_status & DmaIntRxCompleted != 0 {
        dma_int_enable &= !DmaIntRxCompleted;
        unsafe { eth_rx_ready(gmacdev) };
    }
    if dma_status & DmaIntTxUnderflow != 0 {
        unsafe { eth_printf(b"gmac transmit underflow\n\0" as *const u8) };
    }
    if dma_status & DmaIntRcvOverflow != 0 {
        unsafe { eth_printf(b"gmac receive underflow\n\0" as *const u8) };
    }
    if dma_status & DmaIntTxNoBuffer != 0 {}
    if dma_status & DmaIntTxStopped != 0 {
        unsafe { eth_printf(b"gmac transmit process stopped\n\0" as *const u8) };
    }
    if dma_status & DmaIntTxCompleted != 0 {
        eth_handle_tx_over(gmacdev);
    }
    eth_dma_enable_interrupt(gmacdev, dma_int_enable);
}

// 初始化
#[unsafe(no_mangle)]
pub extern "C" fn eth_init(gmacdev: &mut net_device) -> i32 {
    // 在eth_init内或外，利用uncached地址初始化结构体的iobase
    // gmacdev.iobase = eth_phys_to_uncached(0x40040000);
    gmacdev.MacBase = gmacdev.iobase + 0x0000;
    gmacdev.DmaBase = gmacdev.iobase + 0x1000;
    gmacdev.PhyBase = 0;
    gmacdev.Version = eth_mac_read_reg(gmacdev.MacBase, GmacVersion);

    eth_dma_reset(gmacdev);
    eth_mac_set_addr(gmacdev);
    eth_phy_init(gmacdev);

    eth_setup_rx_desc_queue(gmacdev, 128);
    eth_setup_tx_desc_queue(gmacdev, 128);

    eth_dma_reg_init(gmacdev);
    eth_gmac_reg_init(gmacdev);

    unsafe { eth_sync_dcache() };

    eth_gmac_disable_mmc_irq(gmacdev);
    eth_dma_clear_curr_irq(gmacdev);
    eth_dma_enable_interrupt(gmacdev, DmaIntEnable);

    eth_gmac_enable_rx(gmacdev);
    eth_gmac_enable_tx(gmacdev);
    eth_dma_enable_rx(gmacdev);
    eth_dma_enable_tx(gmacdev);

    unsafe { eth_isr_install() };

    return 0;
}
