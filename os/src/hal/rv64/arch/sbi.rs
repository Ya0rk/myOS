use core::arch::asm;
use sbi_spec::time::{EID_TIME, SET_TIMER};
// use sbi_spec::srst::{EID_SRST, SYSTEM_RESET};
use sbi_spec::hsm::EID_HSM;

const RET_SUCCESS: usize = 0;
const ARG_SET_TIMER: (usize, usize) = (EID_TIME, SET_TIMER);
// const ARG_SHUTDOWN: (usize, usize) = (EID_SRST, SYSTEM_RESET);
const ARG_HART_START: (usize, usize) = (EID_HSM, 0);

macro_rules! sbi_call {
    (($eid:expr, $fid:expr): ($eid_ty:ty, $fid_ty:ty), $arg0:expr, $arg1:expr, $arg2:expr) => {{
        let mut ret: usize;
        unsafe {
            asm!(
                "ecall",
                inlateout("a0") $arg0 => ret,
                in("a1") $arg1,
                in("a2") $arg2,
                in("a6") $fid,
                in("a7") $eid,
            );
        }
        ret
    }};
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_call!((ARG_SET_TIMER.0, ARG_SET_TIMER.1): (usize, usize), timer, 0, 0);
}

/// use sbi call to start the specific core
pub fn hart_start(hartid: usize, start_addr: usize) -> bool {
    sbi_call!((ARG_HART_START.0, ARG_HART_START.1): (usize, usize), hartid, start_addr, 0) == RET_SUCCESS
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    // sbi_call!((ARG_SHUTDOWN.0, ARG_SHUTDOWN.1): (usize, usize), 0 as usize, 0, 0);
    // panic!("shutdown error: please check your code!");
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}