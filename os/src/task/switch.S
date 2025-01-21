.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save kernel stack of current task
    sd sp, 8(a0)
    # save ra & s0~s11 of current execution
    sd ra, 0(a0)

    # 保存 s0-s11
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr

    # 下一个task的地址
    ld ra, 0(a1)

    # 下一个task的寄存器
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    
    # restore kernel stack of next task
    ld sp, 8(a1)
    ret

