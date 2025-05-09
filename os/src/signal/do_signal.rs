use core::alloc::Layout;
use alloc::sync::Arc;
use log::info;
use crate::{hal::trap::__sigret_helper, mm::translated_byte_buffer, signal::{SigActionFlag, SigHandlerType, SigNom, UContext, SIG_DFL, SIG_IGN}, task::TaskControlBlock
};

/// 这里包含了所有默认的信号处理方式

pub fn do_signal(task:&Arc<TaskControlBlock>) {
    let trap_cx = task.get_trap_cx_mut();
    let all_len = task.sig_pending.lock().len();
    let mut cur = 0;

    while let Some(siginfo) = task.sig_pending.lock().take_one() {
        let old_sigmask = task.get_blocked();
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

        let sig_handler = task.handler.lock().fetch_signal_handler(signo).sa;

        info!("[do_signal] find a signal: {} ." , signo);

        // if intr && action.flags.contains(SigActionFlag::SA_RESTART) {
        //     cx.sepc -= 4;
        //     cx.restore_last_user_a0();
        //     log::info!("[do_signal] restart syscall");
        //     intr = false;
        // }
        match sig_handler.sa_handler {
            SIG_IGN => {}
            SIG_DFL => { default_func(task, siginfo.signo); }
            handler => {
                // 如果没有SA_NODEFER，在执行当前信号处理函数期间，自动阻塞当前信号
                if !sig_handler
                    .sa_flags
                    .contains(SigActionFlag::SA_NODEFER) {
                        task.get_blocked_mut().set_sig(signo);
                }

                // 可能有其他信号也需要阻塞
                *task.get_blocked_mut() |= sig_handler.sa_mask;
                trap_cx.float_regs.save();

                let old_sp = trap_cx.get_sp();
                // 指向ucontext地址
                let new_sp = old_sp - Layout::new::<UContext>().pad_to_align().size();
                task.set_ucontext(new_sp);

                let sig_stack = task.get_sig_stack_mut().take();
                let token = task.get_user_token();
                // 保存当前的user 状态,在sigreturn中恢复
                let ucontext = UContext::new(old_sigmask, sig_stack, &trap_cx);
                copy2user(token, new_sp as *mut UContext, &ucontext);

                // 修改trap_cx，函数trap return后返回到信号处理函数
                trap_cx.flash(handler, new_sp, __sigret_helper as usize, signo);

            }
        }

    }
    task.set_pending(!task.sig_pending.lock().is_empty());
}

/// 根据signo分发处理函数
fn default_func(task: &Arc<TaskControlBlock>, signo: SigNom) {
    info!("[default_func] signo = {}", signo as i32);
    match signo {
        SigNom::SIGCHLD | SigNom::SIGURG  | SigNom::SIGWINCH => {}, // no Core Dump
        SigNom::SIGSTOP | SigNom::SIGTSTP | SigNom::SIGTTIN | SigNom::SIGTTOU => do_signal_stop(task, signo),   // no core dump
        SigNom::SIGCONT => do_signal_continue(task, signo),         // no core dump
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

/// 从Pantheon借鉴, 不知道可不可以使用UserBuffer
/// Copy data from `src` out of kernel memory set into `dst` which lives in the
/// given memory set indicated by the given `token`.
pub fn copy2user<T>(token: usize, dst: *mut T, src: &T) {
    let mut dst_buffer =
        translated_byte_buffer(token, dst as *const u8, core::mem::size_of::<T>());

    let src_slice = unsafe {
        core::slice::from_raw_parts(src as *const T as *const u8, core::mem::size_of::<T>())
    };
    let mut index = 0;

    let mut start_byte = 0;
    loop {
        let dst_slice = &mut dst_buffer[index];
        index += 1;
        let dst_slice_len = dst_slice.len();
        dst_slice.copy_from_slice(&src_slice[start_byte..start_byte + dst_slice_len]);
        start_byte += dst_slice_len;
        if dst_buffer.len() == index {
            break;
        }
    }
}