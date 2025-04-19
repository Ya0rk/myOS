# 将initproc和onlinetests提前放到内存里
    .section .data
    .globl initproc_start
    .globl initproc_end
    .align 3
initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/initproc"
initproc_end:

    .section .data
    .globl user_shell_start
    .globl user_shell_end
    .align 3
user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/user_shell"
user_shell_end:


    .section .data
    .globl busybox_start
    .globl busybox_end
    .align 3
busybox_start:
	.incbin "../user/target/riscv64gc-unknown-none-elf/release/mybusybox"
busybox_end: