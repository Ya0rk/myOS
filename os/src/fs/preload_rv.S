# # 将initproc和onlinetests提前放到内存里
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
    .globl mbshell_start
    .globl mbshell_end
    .align 3
mbshell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/mbshell"
mbshell_end:

    .section .data
    .globl gbshell_start
    .globl gbshell_end
    .align 3
gbshell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/gbshell"
gbshell_end:

    .section .data
    .globl autorun_start
    .globl autorun_end
    .align 3
autorun_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/autorun"
autorun_end:

    .section .data
    .globl test_initproc_start
    .globl test_initproc_end
    .align 3
test_initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/test_initproc"
test_initproc_end:

