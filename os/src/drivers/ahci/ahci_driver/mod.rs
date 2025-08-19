#![no_std]
#![allow(dead_code, unused_assignments, unused_mut)]

pub mod drv_ahci;
pub mod libahci;
pub mod libata;
pub mod platform;

// use core::panic::PanicInfo;

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }
