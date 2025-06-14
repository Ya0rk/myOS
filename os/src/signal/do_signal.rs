use crate::{
    hal::trap::__sigret_helper,
    mm::translated_byte_buffer,
    signal::{LinuxSigInfo, SigActionFlag, SigHandlerType, SigNom, UContext, SIG_DFL, SIG_IGN},
    task::TaskControlBlock,
};
use alloc::sync::Arc;
use core::alloc::Layout;
use log::info;

/// 这里包含了所有默认的信号处理方式

pub fn do_signal(task: &Arc<TaskControlBlock>) {
    let trap_cx = task.get_trap_cx_mut();
    let all_len = task.sig_pending.lock().len();
    let mut cur = 0;
    let old_sigmask = *task.get_blocked();

    loop {
        let siginfo = {
            match task.sig_pending.lock().take_one(old_sigmask) {
                Some(siginfo) => siginfo,
                None => {
                    break;
                }
            }
        };
        cur += 1;
        // 避免队列中全是被阻塞的信号，造成死循环
        if cur > all_len {
            break;
        }
        // 被阻塞的信号需要跳过，注意SIGKILL和SIGSTOP不能被屏蔽
        let signo = siginfo.signo as usize;
        if old_sigmask.have(signo)
            && signo != SigNom::SIGKILL as usize
            && signo != SigNom::SIGSTOP as usize
        {
            // 再将信号重新放回队列
            task.sig_pending.lock().add(siginfo);
            continue;
        }

        let k_action = task.handler.lock().fetch_signal_handler(signo);
        let sig_action = k_action.sa;
        info!(
            "[do_signal] task id = {}, find a signal: {}, handler = {:#x}, flags = {:?}.",
            task.get_pid(),
            signo,
            sig_action.sa_handler,
            sig_action.sa_flags
        );
        // if sig_action.sa_flags.contains(SigActionFlag::SA_RESTART) {
        //     info!("[do_signal] restart");
        //     trap_cx.sepc -= 4;
        //     trap_cx.restore_last_a0();
        // }

        match k_action.sa_type {
            SigHandlerType::IGNORE => {}
            SigHandlerType::DEFAULT => {
                default_func(task, siginfo.signo);
            }
            SigHandlerType::Customized { handler } => {
                // 如果没有SA_NODEFER，在执行当前信号处理函数期间，自动阻塞当前信号
                if !sig_action.sa_flags.contains(SigActionFlag::SA_NODEFER) {
                    task.get_blocked_mut().insert_sig(signo);
                }

                // 可能有其他信号也需要阻塞
                *task.get_blocked_mut() |= sig_action.sa_mask;
                trap_cx.float_regs.save();

                let old_sp = trap_cx.get_sp();

                let sig_stack = task.get_sig_stack_mut().take();
                let mut new_sp = match sig_stack {
                    Some(sig_stack) => {
                        // 用户自定义的栈
                        info!("[do_signal] user define stack");
                        let mut new_sp = sig_stack.ss_sp + sig_stack.ss_size;
                        new_sp -= size_of::<UContext>();
                        new_sp
                    }
                    None => {
                        // 普通栈
                        info!("[do_signal] default stack");
                        old_sp - size_of::<UContext>()
                    }
                };
                // 将ucontext指针保存在tcb中
                task.set_ucontext(new_sp);

                let token = task.get_user_token();
                // 保存当前的user 状态,在sigreturn中恢复
                // 包括本来的sepc，后序在sigreturn中恢复
                let mut ucontext = UContext::new(old_sigmask, sig_stack, &trap_cx);
                ucontext.uc_mcontext.user_gp.zero = trap_cx.sepc;
                // 将ucontext拷贝到用户栈中
                unsafe { core::ptr::write(new_sp as *mut UContext, ucontext) };

                if sig_action.sa_flags.contains(SigActionFlag::SA_SIGINFO) {
                    // 若信号处理函数通过 sigaction 注册时设置了此标志，表示处理函数需要接收以下参数：
                    // void handler(int sig, siginfo_t *info, void *ucontext);
                    // a0(x10): 信号编号,在后面的trap_cx.flash中设置
                    // a1(x11): 信号信息结构体指针
                    // a2(x12): ucontext 结构体指针
                    trap_cx.user_gp.a2 = new_sp; // a2
                    let mut siginfo_v = LinuxSigInfo::new(signo as i32, siginfo.sigcode as i32);
                    new_sp -= size_of::<LinuxSigInfo>();
                    // 将siginfo_v拷贝到用户栈中
                    unsafe { core::ptr::write(new_sp as *mut LinuxSigInfo, siginfo_v) };
                    trap_cx.user_gp.a1 = new_sp; // a1
                }

                let mut gp: usize;

                // loongarch has no global pointer reg
                #[cfg(target_arch = "riscv64")]
                {
                    gp = ucontext.get_user_gp().gp; // gp
                }
                #[cfg(target_arch = "loongarch64")]
                {
                    gp = 0;
                }
                let tp = ucontext.get_user_gp().tp; // tp

                // 修改trap_cx，函数trap return后返回到用户自定义的函数，
                // 自定义函数执行完后返回到sigreturn,这里的__sigret_helper是一个汇编，触发sigreturn
                // 这里的sigreturn是一个系统调用，返回到内核态
                info!("[do_signal] before flash");
                trap_cx.flash(handler, new_sp, __sigret_helper as usize, signo, gp, tp);
                // break;
            }
        }
    }
    task.set_pending(!task.sig_pending.lock().is_empty());
}

/// 根据signo分发处理函数
fn default_func(task: &Arc<TaskControlBlock>, signo: SigNom) {
    info!("[default_func] signo = {:?}", signo);
    match signo {
        // TODO(YJJ):有待完善
        SigNom::SIGCHLD | SigNom::SIGURG | SigNom::SIGWINCH => {} // no Core Dump
        SigNom::SIGSTOP | SigNom::SIGTSTP | SigNom::SIGTTIN | SigNom::SIGTTOU => {
            do_signal_stop(task, signo)
        } // no core dump
        SigNom::SIGCONT => do_signal_continue(task, signo),       // no core dump
        _ => do_group_exit(task, signo),
    }
}

/// 沙西所有子线程
fn do_group_exit(task: &Arc<TaskControlBlock>, signo: SigNom) {
    task.kill_all_thread();
    info!("[do_goroup_exit] signo = {}", signo as i32);
    task.set_exit_code(signo as i32);
}

/// 在 Linux 中，当子进程状态变化时，内核会向父进程发送 SIGCHLD 信号，
/// 并通过 siginfo_t 结构体中的 si_code 字段告知具体事件类型。
fn do_signal_stop(task: &Arc<TaskControlBlock>, signo: SigNom) {
    // 挂起子进程并通知父进程
    task.stop_all_thread(signo);
}

/// 在 Linux 中，当子进程状态变化时，内核会向父进程发送 SIGCHLD 信号，
/// 并通过 siginfo_t 结构体中的 si_code 字段告知具体事件类型。
fn do_signal_continue(task: &Arc<TaskControlBlock>, signo: SigNom) {
    // 唤醒子进程并通知父进程
    task.cont_all_thread(signo);
}
