.section .text
    .globl __trap_from_user
    .globl __return_to_user
    .global __trap_from_kernel
    .align 2

__trap_from_user:

__return_to_user:

__trap_from_kernel:

.section .text.sigret
.align 12
.global __sigret_helper
__sigret_helper: