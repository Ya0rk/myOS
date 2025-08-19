#![no_std]
#![allow(dead_code, unused_assignments, unused_mut)]

mod drv_eth;
mod eth_defs;
mod eth_dev;
mod platform;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
