//！ `hal/la/trap/kernel_trap.rs`

use log::info;
use loongarch64::register::*;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::estat::{Exception, Interrupt, Trap};

use crate::mm::memory_space::PageFaultAccessType;
use crate::task::current_task;

#[no_mangle]
pub fn kernel_trap_handler() {
    // info!("kernel trap");
    let estat = estat::read();
    let crmd = crmd::read();
    let era = era::read();
    if crmd.plv() != CpuMode::Ring0 {
        // 只有在内核态才会触发中断
        panic!("{:?}", estat.cause());
    }
    match estat.cause() {
        Trap::Interrupt(Interrupt::Timer) => {
            // 清除时钟专断
            // info!("timer interrupt from kernel");
            ticlr::clear_timer_interrupt();
        }
        Trap::Interrupt(Interrupt::HWI0) => {
            // 中断0 --- 外部中断处理
            unimplemented!("loongarch64 Trap::Interrupt(Interrupt::HWI0)");
        }
        Trap::Exception(e) => {
            match e {
                // Exception::Syscall => {
                //     let mut cx = current_trap_cx();
                //     let syscall_id = cx.user_x[11];
                //     let result = syscall(
                //         syscall_id,
                //         [cx.user_x[4],
                //         cx.user_x[5],
                //         cx.user_x[6],
                //         cx.user_x[7],
                //         cx.user_x[8],
                //         cx.user_x[9]]
                //     ).await;
        
                //     cx = current_trap_cx();
        
                //     match result {
                //         Ok(ret) => {
                //             cx.user_x[4] = ret as usize;
                //         }
                //         Err(err) => {
                //             if err as isize == -1 {
                //                 cx.user_x[4] = err as usize;
                //             } else {
                //                 cx.user_x[4] = -(err as isize) as usize;
                //                 info!("[syscall ret] sysID = {}, errmsg: {}", syscall_id, err.get_info());
                //             }
                //         }
                //     }
                // }
        
                Exception::LoadPageFault |
                Exception::StorePageFault |
                Exception::FetchPageFault |
                Exception::PageModifyFault |
                Exception::PageNonReadableFault |
                Exception::PageNonExecutableFault => {
                    let va = badv::read().vaddr();  
                    info!("[kernel_trap_handler] meet a pagefault {:?} at {:#x}", e, va);
                    let access_type = match e {
                        Exception::LoadPageFault | Exception::PageNonReadableFault => {
                            PageFaultAccessType::RO
                        }
                        Exception::StorePageFault | Exception::PageModifyFault => {
                            PageFaultAccessType::RW
                        }
                        Exception::FetchPageFault | Exception::PageNonExecutableFault => {
                            PageFaultAccessType::RX
                        }
                        _=> {
                            unreachable!()
                        }
                    };
                    
                    current_task().unwrap().with_mut_memory_space(|m| {
                        m.handle_page_fault(va.into(), access_type)
                    });
                }
        
                _ => {
                    panic!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
                }
            }
        }
        _ => {
            panic!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
        }
    }
    era::set_pc(era.pc());
    // info!("kernel trap end");
}