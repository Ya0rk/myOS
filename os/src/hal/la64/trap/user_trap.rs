#![allow(unused_import_braces)]
#![allow(unused)]
use super::{__return_to_user, set_trap_handler, IndertifyMode};
use crate::hal::arch::sstatus::SPP;
use crate::sync::{disable_interrupt, set_next_trigger, yield_now};
use crate::syscall::syscall;
use crate::task::{current_task, current_trap_cx, executor, get_current_hart_id};
use core::arch::asm;
use log::info;


#[cfg(target_arch = "loongarch64")]
#[no_mangle]
pub async fn user_trap_handler() {
    use log::error;
    use loongarch64::{
        asm,
        register::{
            badi, badv, crmd, era,
            estat::{self, Exception, Interrupt, Trap},
            ticlr, CpuMode,
        },
    };

    use crate::{mm::memory_space::PageFaultAccessType, sync::TIMER_QUEUE, task::get_current_cpu};
    let estat = estat::read();
    let crmd = crmd::read();
    let era = era::read();
    // println!("[user_trap_handler] cause:{:?}, crmd:{:?}, era:{:#x}", estat.cause(), crmd, era.pc());
    let task = current_task().unwrap();
    if crmd.plv() != CpuMode::Ring0 {
        // 只有在内核态才会触发中断
        panic!("{:?}", estat.cause());
    }
    match estat.cause() {
        Trap::Interrupt(Interrupt::Timer) => {
            // 清除时钟专断
            // info!("timer interrupt from kernel");
            TIMER_QUEUE.handle_expired();
            set_next_trigger();
            yield_now().await;
        }
        Trap::Interrupt(Interrupt::HWI0) => {
            // 中断0 --- 外部中断处理
            unimplemented!("loongarch64 Trap::Interrupt(Interrupt::HWI0)");
        }

        Trap::Exception(e) => {
            match e {
                Exception::Syscall => {
                    era::set_pc(era.pc() + 4);

                    let mut cx = current_trap_cx();
                    cx.sepc += 4;
                    let syscall_id = cx.user_gp.a7;
                    let args  = [cx.user_gp.a0,
                                cx.user_gp.a1,
                                cx.user_gp.a2,
                                cx.user_gp.a3,
                                cx.user_gp.a4,
                                cx.user_gp.a5];
                    // info!("[user_trap_handler] syscall id:{}, args:{:?}", syscall_id, args);
                    let result = syscall(syscall_id, args).await;

                    cx = current_trap_cx();

                    match result {
                        Ok(ret) => {
                            cx.user_gp.a0 = ret as usize;
                            // info!("[syscall ret] OK:{}", ret);
                        }
                        Err(err) => {
                            if err as isize == -1 {
                                cx.user_gp.a0 = err as usize;
                                // info!("[syscall ret] Err:{:?}", err);
                            } else {
                                cx.user_gp.a0 = -(err as isize) as usize;
                                // info!("[syscall ret] sysID = {}, errmsg: {}", syscall_id, err.get_info());
                            }
                        }
                    }
                }

                Exception::LoadPageFault
                | Exception::StorePageFault
                | Exception::FetchPageFault
                | Exception::PageModifyFault
                | Exception::PageNonReadableFault
                | Exception::PageNonExecutableFault => {
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
                        _ => {
                            unreachable!()
                        }
                    };

                    let va = badv::read().vaddr();
                    // if (va == 0) {
                    //     panic!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
                    // }
                    current_task()
                        .unwrap()
                        .with_mut_memory_space(|m| m.handle_page_fault(va.into(), access_type))
                        .unwrap_or_else(|e| {
                            use log::error;
                            task.set_zombie();
                            error!(
                                "{:?} pc: {:#x} BADV: {:#x}",
                                estat.cause(),
                                era.pc(),
                                badv::read().vaddr()
                            );
                        });
                }
                Exception::InstructionNotExist => {
                    error!(
                        "{:?} pc: {:#x} BADV: {:#x}",
                        estat.cause(),
                        era.pc(),
                        badv::read().vaddr()
                    );
                    unsafe {
                        let pc = era.pc() as *const usize;
                        info!("[user_trap_handler] inst: {:b}", *pc);
                        panic!("InstructionNotExist");
                    }
                }
                _ => {
                    panic!(
                        "Cause:{:?} ecode:{:#x} is:{:#x} pc: {:#x} BADV: {:#x} BADI: {:#x}",
                        estat.cause(),
                        estat.ecode(),
                        estat.is(),
                        era.pc(),
                        badv::read().vaddr(),
                        badi::read().inst()
                    );
                }
            }
        }

        _ => {
            unsafe {
                let mut fcsr0 = current_trap_cx().float_regs.fcsr;
                // asm!("movfcsr2gr {}, $fcsr0", out(reg) fcsr0);
                error!("[user_trap_handler] fcsr: {:#b}", fcsr0);
            }
            panic!(
                "Cause:{:?} ecode:{:#x} is:{:#x} pc: {:#x} BADV: {:#x} BADI: {:#x}",
                estat.cause(),
                estat.ecode(),
                estat.is(),
                era.pc(),
                badv::read().vaddr(),
                badi::read().inst()
            );
        }
    }
    // era::set_pc(era.pc());
    // info!("kernel trap end");
}

#[no_mangle]
pub fn user_trap_return() {
    use crate::task::get_current_cpu;
    // 重新修改stvec设置 user 的trap handler entry
    // set_trap_handler(IndertifyMode::User);

    let trap_cx = current_trap_cx();
    // trap_cx.float_regs.trap_out_do_with_freg();
    // info!("[user_trap_return] 1");
    trap_cx.sstatus.set_spp(SPP::User);
    trap_cx.sstatus.set_pie(true);

    // disable_interrupt();
    let task = current_task().unwrap();

    get_current_cpu().timer_irq_reset();
    task.get_time_data_mut().set_trap_out_time();
    // info!("[user_trap_return] entering __return_to_user, cx:{:?}", *trap_cx);

    unsafe {
        __return_to_user(trap_cx);
    }
    // info!("[user_trap_return] entering trap");
    task.get_time_data_mut().set_trap_in_time();

    // trap_cx.float_regs.trap_in_do_with_freg(trap_cx.sstatus);
}
