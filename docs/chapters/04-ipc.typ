#import "../template.typ": img, tbl

= 进程间通信

== 信号机制

信号是操作系统向进程传递事件通知的一种机制，主要用于通知进程发生了异步事件。Phoenix 与往届作品 Titanix 的信号机制相比，信号机制更加完善。Titanix 的信号队列中只有信号编号，Phoenix 参考了 Linux 的实现，使用`SigInfo`结构体代替，除了能表示信号编号外，还能携带更多的信息：

```rs
pub struct SigInfo {
    pub sig: Sig,
    pub code: i32,
    pub details: SigDetails,
}
```

这种设计与 POSIX 标准中的 `siginfo_t` 结构体相似，增强了 Phoenix 系统与标准 POSIX 接口的兼容性。并且使得 Phoenix 的信号机制具备了更强的表达能力和灵活性。携带附加信息可以是信号的来源、产生原因和相关的上下文数据。例如，`code` 字段可以用于区分不同类型的信号或事件，`details` 字段则可以包含更详细的上下文信息：

```rs
pub enum SigDetails {
    None,
    Kill {
        /// sender's pid
        pid: usize,
    },
    CHLD {
        /// which child
        pid: usize,
        /// exit code
        status: i32,
        utime: Duration,
        stime: Duration,
    },
}
```
例如，当子进程状态变为`Zombie`时需要向父进程发送`SIGCHLD`信号告知父进程状态改变，此时将`SigInfo`中的`SigDetails`字段设为`CHLD`，并且告诉父进程自己的`pid`、状态码`status`、用户态运行时间`utime`和内核态运行时间`stime`，这些信息可以有助于父进程在`Wait4`系统调用时掌握子进程的数据。

=== 信号处理函数

在任务由内核态返回到用户态之前，往往需要执行信号处理函数，检查和处理挂起的信号，调用适当的信号处理程序或执行默认行为，确保进程能够正确响应和处理信号。这一机制是操作系统处理异步事件、进程控制和进程间通信的重要组成部分。

这里涉及到一个细节，就是如果此时信号待处理队列中有多个需要处理的信号同时到来，那么应该以什么顺序处理。大部分往届作品都是按照信号到来顺序依次处理的，这种 FIFO（先入先出）方法虽然简单直接，但在处理高优先级和紧急事件时可能存在不足。例如，当一个需要立即响应的紧急信号和一个普通信号同时到达时，按照到达顺序处理可能导致紧急信号的响应延迟。Phoenix 在设计信号处理机制时，参考了 Linux 内核的实现，引入了信号优先级的概念，这样的好处是紧急信号可以立即得到处理，减少了响应延迟，提升了系统对关键事件的响应能力。例如，当发生非法内存访问时，会触发`SIGSEGV`（Segmentation Fault）信号，指示程序运行中出现严重问题，这种信号在 Phoenix 中会优先处理。

用户可以使用`sigaction`系统调用为某些信号自定义信号处理函数，操作系统内核会按照如下步骤来调用用户自定义的信号处理函数：
+ 保存上下文：内核会保存当前的进程执行上下文，包括寄存器状态、堆栈指针、程序计数器等，以便在信号处理完成后恢复进程的执行。
+ 切换到用户态：内核切换到用户态，并开始执行用户自定义的信号处理函数
+ 恢复上下文：使用 `sigreturn` 函数返回到内核态并且恢复之前保存的进程上下文，如寄存器状态、堆栈指针、程序计数器等。

这里同样存在一个问题，即切换到用户态需要保存上下文，那么应该将上下文保存在哪里？部分作品如往届一等奖作品 Titanix 将上下文记录在内核态，这当然也可以，但是对于 POSIX 规范中一些标志位就难以支持，例如设置了`SA_SIGINFO`标志位的信号处理程序的函数签名会变成如下形式：

```c
void handler(int sig, siginfo_t *info, void *ucontext);
```

这里`ucontext`是指向`ucontext_t`结构体的指针，提供接收信号时进程的上下文信息，如果将进程上下文保存在了内核态，那么用户将有机会访问到内核中的信息，这无疑是非常不安全的。虽然竞赛并未有测试程序需要用到该标志位，但是 Phoenix 的目标是实现符合 POSIX 规范的操作系统，因此依然实现了该标志位。

Phoenix 参考了 Linux 的设计，将信号处理时需要保存的进程上下文保存在了用户栈中，这样`ucontext`指向用户栈就非常安全。

=== 系统调用的打断与恢复

在往届参赛作品中，几乎都不支持被含有`SA_RESTART`标志的信号打断的系统调用的恢复功能。在类Unix操作系统中，如果慢系统调用（如 `sys_read`、`sys_pselect6`）在执行期间被信号打断并且该信号的处理程序（signal handler）被触发，那么：
+ #strong[带有 `SA_RESTART` 标志位的信号]：信号处理程序返回后，该系统调用会被自动重新启动，而不是直接返回错误。这对某些慢系统调用（如 read、write、select、pselect 等）尤为重要，因为这些调用可能会因为等待外部事件而阻塞很长时间。
+ #strong[没有 `SA_RESTART` 标志位的信号]：系统调用会被打断并返回一个错误代码，通常是 EINTR（表示系统调用被中断）。调用者需要检查返回值并决定是否重新执行该系统调用

Phoenix 的设计目标是实现功能完善的操作系统，因此支持了被打断的系统调用的恢复功能。

Phoenix采用如下的方案：如果是系统调用陷入内核，并且系统调用返回的是`SysError::EINTR`错误，`trap_handler`函数会返回`true`表示系统调用被信号打断了，
```rs
pub async fn trap_handler(task: &Arc<Task>) -> bool {
    /* skip */
    match cause {
        Trap::Exception(e) => {
            match e {
                Exception::UserEnvCall => {
                    let syscall_no = cx.syscall_no();
                    cx.set_user_pc_to_next();
                    // get system call return value
                    let ret = Syscall::new(task)
                        .syscall(syscall_no, cx.syscall_args())
                        .await;
                    cx.save_last_user_a0();
                    cx.set_user_a0(ret);
                    if ret == -(SysError::EINTR as isize) as usize {
                        return true;
                    }
                }
                /* skip */
            }
        }
        /* skip */
    }
    false
}
```
`trap_handler`函数的返回值会传递给`do_signal`函数，在`do_signal`中检测信号是否含有`SA_RESTART`标志位，如果发现需要重启系统调用，会将`TrapContext`中的`sepc`寄存器-4，表示重新执行，由于用户自定义的信号处理程序`signal handler`函数执行完的返回值会将`a0`寄存器覆盖掉，这里使用`restore_last_user_a0`函数进行备份。
```rs
pub fn do_signal(task: &Arc<Task>, mut intr: bool) -> SysResult<()> {
    /* skip */

    while /* skip */ {
        let action = /* skip */;
        if intr && action.flags.contains(SigActionFlag::SA_RESTART) {
            cx.sepc -= 4;
            cx.restore_last_user_a0();
            intr = false;
        }
        /* skip */
    }
    /* skip */
}
```