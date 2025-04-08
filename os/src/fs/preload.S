# 将initproc和onlinetests提前放到内存里
    .section .data
    .global initproc_start
    .global initproc_end
    .global user_shell_start
    .global user_shell_end
    .align 12
initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/initproc"
initproc_end:
    .align 12
user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/user_shell"
user_shell_end:
