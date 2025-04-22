# 目前仅支持单核启动
    .section .text.entry
    .globl _start
_start:
    ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
    lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
    csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
    ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
    lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
    csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

    # Enable PG 
    li.w        $t0, 0xb0       # PLV=0, IE=0, PG=1
    csrwr       $t0, 0x0        # LOONGARCH_CSR_CRMD
    li.w        $t0, 0x00       # PLV=0, PIE=0, PWE=0
    csrwr       $t0, 0x1        # LOONGARCH_CSR_PRMD
    li.w        $t0, 0x00       # FPE=0, SXE=0, ASXE=0, BTE=0
    csrwr       $t0, 0x2        # LOONGARCH_CSR_EUEN

    la.abs      $sp, boot_stack_top
    csrrd       $a0, 0x20           # cpuid
    b           rust_main

    .section .bss.stack
    .align 16
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16 * 2
    .globl boot_stack_lower_bound
boot_stack_top:

.section .data
boot_pagetable:
