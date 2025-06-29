#import "../components/prelude.typ": *

= 中断与异常处理

== 特权级切换

中断与异常是用户态与内核态之间切换的中重要机制。在Del0n1x中，为了更加清晰的设计模式，将中断异常分为两类：

#list(
    [用户中断异常：发生在用户态(U-Mode)的中断或异常],
    [内核中断异常：发生在内核态(S-Mode)的中断或异常],
    indent: 2em
)
//   - 用户中断异常：发生在用户态(U-Mode)的中断或异常

//   - 内核中断异常：发生在内核态(S-Mode)的中断或异常

#h(2em)当用户态发生中断或异常时，系统需要完成从用户态到内核态的切换。这个过程通过汇编函数 ` __trap_from_user` 实现，这是用户态中断处理的入口点，负责保存完整的用户上下文。而sepc（era）、stval（estat）等寄存器则是由硬件自动完成保存。用户上下文保存在如下结构中：

#code-figure(
```rs
pub struct TrapContext {
    /* 0-31 */ pub user_gp: GPRegs,
    /*  32  */ pub sstatus: Sstatus,
    /*  33  */ pub sepc: usize,
    /*  34  */ pub kernel_sp: usize,
    /*  35  */ pub kernel_ra: usize,
    /* 36-47*/ pub kernel_s: [usize; 12],
    /*  48  */ pub kernel_fp: usize,
    /*  49  */ pub kernel_tp: usize,
    /*  50  */ pub float_regs: UserFloatRegs,
}
```,
    caption: [TrapContext结构],
    label-name: "TrapContext-struct",
)

#h(2em)需要注意的是，在riscv64中，我们通过检测fs寄存器是否被使用决定是否保存浮点寄存器，这样的设计减少上下文切换的开销。但是通过查阅手册，loongarch64架构中并没有提供这样的寄存器，所以我们将loongarch64中浮点寄存器的保存处理放置在`__trap_from_user`，同时我们在TrapContext中增加了`fcsr`字段用于保存浮点控制状态寄存器。`fcsr`是浮点运算单元（FPU）的核心控制寄存器，它负责管理浮点运算的异常标志、舍入模式、使能控制等关键功能。

== 处理过程

=== 内核中断异常处理

以下均以riscv64为例。Del0n1x的内核态中断异常处理目前支持时钟中断和地址缺页异常（对于外部中断放置于比赛下一阶段进行完善） ，代码如下：

#code-figure(
```rs
Trap::Interrupt(Interrupt::SupervisorTimer) => {
    TIMER_QUEUE.handle_expired();
    get_current_cpu().timer_irq_inc();
    set_next_trigger();
}
Trap::Exception(e) => match e {
    Exception::StorePageFault
    | Exception::InstructionPageFault
    | Exception::LoadPageFault => {
        let access_type = match e {
            Exception::InstructionPageFault => PageFaultAccessType::RX,
            Exception::LoadPageFault => PageFaultAccessType::RO,
            Exception::StorePageFault => PageFaultAccessType::RW,
            _ => unreachable!(),
        };

        let task = current_task().unwrap();
        result = task
            .with_mut_memory_space(|m| {
                m.handle_page_fault(stval.into(), access_type)
            });
        result.is_err().then(|| {
            use crate::hal::arch::current_inst_len;
            sepc::write(sepc + current_inst_len());
        });
    }

set_ktrap_ret(result);
```,
    caption: [内核中断异常处理关键函数],
    label-name: "kernel_trap_handler",
)

内核缺页异常通常发生在系统调用中，由于进程虚拟地址空间没有实际分配独立的物理页帧，而是将其指向父进程对应的物理页帧，当内核向用户传入的地址写入数据时，会触发 PageFault 异常跳转至内核处理函数中，在这次trap的处理中，我们不仅对子进程分配实际的物理页帧，恢复相应页表项的标志位，同时通过检查trap的返回值实现用户地址空间可写性的检查。

=== 用户中断异常处理

在Del0n1x中除了对用户态的 PageFault 处理之外，还实现了时钟中断和系统调用处理。对于时钟中断，Del0n1x检查了全局定时器中是否有超时任务，然后设置下一次时钟中断时间点，最后需要调用 yield 释放当前任务对CPU的使用权，调度下一个任务，避免任务长时间占用CPU导致其他任务饥饿。对于系统调用处理，会先将中断上下文中的sepc 加 4，使得从内核态返回到用户态后能够跳转到下一条指令。然后，调用syscall 函数系统调用。系统调用完成后，将返回值保存在x10 寄存器。

#code-figure(
```rs
....
Trap::Exception(Exception::UserEnvCall) => { // 7
  let mut cx = current_trap_cx();
  let old_sepc: usize = cx.get_sepc();
  let syscall_id = cx.user_gp.a7;
  cx.set_sepc(old_sepc + 4);

  let result = syscall(
      syscall_id, 
      [cx.user_gp.a0, 
      cx.user_gp.a1, 
      cx.user_gp.a2, 
      cx.user_gp.a3, 
      cx.user_gp.a4,
      cx.user_gp.a5]
  ).await;

  // cx is changed during sys_exec, so we have to call it again
  cx = current_trap_cx();

  match result {
      Ok(ret) => {
          cx.user_gp.a0 = ret as usize;
      }
      Err(err) => {
          if (err as isize) < 0 {
              cx.user_gp.a0 = err as usize;
          } else {
              cx.user_gp.a0 = (-(err as isize)) as usize;
          }
      }
  }
}

Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
    TIMER_QUEUE.handle_expired();
    set_next_trigger();
    yield_now().await;
}
....
```,
    caption: [用户中断异常处理函数],
    label-name: "user_trap_handler",
)

=== 返回用户态

Del0n1x中从内核态返回到用户态过程交付予`user_trap_return`函数处理。在该函数的处理逻辑中，首先要通过设置`stvec`寄存器确保下次用户中断的入口地址正确，然后恢复用户浮点寄存器状态，同时修改进程时间戳记录进程`trap_out`时间。最后通过`__reture_to_user`调用`sret`实现 S 监督模式到 U 用户模式特权级的切换。

#code-figure(
```rs
pub fn user_trap_return() {
    // 重新修改stvec设置 user 的trap handler entry
    set_trap_handler(IndertifyMode::User);

    let trap_cx = current_trap_cx();
    trap_cx.float_regs.trap_out_do_with_freg();
    trap_cx.sstatus.set_fs(FS::Clean);

    get_current_cpu().timer_irq_reset();
    let task = current_task().unwrap();
    task.get_time_data_mut().set_trap_out_time();
    unsafe {
        __return_to_user(trap_cx);
    }
    task.get_time_data_mut().set_trap_in_time();

    trap_cx.float_regs.trap_in_do_with_freg(trap_cx.sstatus);
}
```,
    caption: [内核->用户切换],
    label-name: "user_trap_return",
)


#pagebreak()  // 强制分页