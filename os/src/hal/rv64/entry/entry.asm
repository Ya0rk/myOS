    .section .text.entry
    .globl _start
_start:
    # 启动时rust sbi将hart_id放在a0中
    mv tp,a0

    # 设置 mstatus.FS 为 "Initial" 状态 (0b01), 
    # 启用浮点寄存器，不然后面汇编不能识别双精度相关指令，会触发
    # li t0, (1 << 13)  # FS 字段位于 bit 13-14
    # csrs mstatus, t0

    # 根据hart_id不同设置kernel stack的sp
    slli t0, a0, 16  # t0 = hart_id << 16(4096 * 16)
    la sp, boot_stack_top
    sub sp, sp, t0  # sp = stack top - hart_id * stack_size

    # 因为现在内核链接地址在 0xffff_ffc0_8020_0000（这是虚拟地址）
    # 所以我们要打开页表机制，这样才能找到正确的物理地址
    # satp: 8 << 60 | boot_pagetable （开启页表机制 三级页表）
    la t0, boot_pagetable
    li t1, 8 << 60
    srli t0, t0, 12
    or t0, t0, t1
    csrw satp, t0
    sfence.vma

    # 调用 辅助函数（在utils/boot.rs中），在辅助函数中调用真正的程序入口
    call jump_helper

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16 * 2 # 根据CPU数量开辟栈空间
    .globl boot_stack_top
boot_stack_top:

.section .data
    .align 12
boot_pagetable:
    # 这是大页表
    # 里面只需要两个pte，供我们找到正确的物理地址
    # 0x0000_0000_8000_0000 -> 0x0000_0000_8000_0000

    .quad 0
    .quad 0
    .quad (0x80000 << 10) | 0xcf # VRWXAD
    .zero 8 * 255
    .quad (0x80000 << 10) | 0xcf # VRWXAD
    .zero 8 * 253