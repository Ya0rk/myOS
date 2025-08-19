#![no_std]
#![allow(dead_code, unused_assignments, unused_mut)]

mod drv_ahci;
mod libahci;
mod libata;
mod platform;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
