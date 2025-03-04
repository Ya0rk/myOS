use log::*;
use crate::{arch::shutdown, task::get_current_hart_id, utils::backtrace};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[kernel] Hart {}, Panicked at {}:{} {}",
            get_current_hart_id(),
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        error!("[kernel] Hart {}, Panicked: {}",
            get_current_hart_id(),
            info.message()
        );
    }
    backtrace();
    shutdown(true)
}
