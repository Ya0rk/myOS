    .section .text.entry
    .globl _start
_start:
0
    # test entry
    addi.d      $t0, $zero,0x11
    csrwr       $t0, 0x180
    la.global   $sp, boot_stack_top

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_lower_bound
boot_stack_top:

.section .data
boot_pagetable:
