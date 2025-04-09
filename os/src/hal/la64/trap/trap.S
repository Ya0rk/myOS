.section .text
    .globl __trap_from_user
    .globl __return_to_user
    .global __trap_from_kernel
    .align 4

FP_START = 32
.altmacro
.macro SAVE_GP n
    st.d $r\n, $sp, \n*8
.endm
.macro LOAD_GP n
    ld.d $r\n, $sp, \n*8
.endm
.macro SAVE_FP n, m
    fst.d $f\n, $sp, \m*8
.endm
.macro LOAD_FP n, m
    fld.d $f\n, $sp, \m*8
.endm

.section .text
    .globl __trap_from_user
    .globl __return_to_user
    .global __trap_from_kernel
    .align 4

.equ CSR_SAVE, 0x30
# 保存sepc
.equ CSR_ERA, 0x6
# sstatus：
.equ CSR_PRMD, 0x1
.equ CSR_PGDL, 0x19
.equ CSR_PGD, 0x1b

# user -> kernel
__trap_from_user:
    #==================REMEMBER TO TURN OFF THE INTERRUPT !=======================
    # 交换sp和CSR_SAVE的值
    csrwr $sp, CSR_SAVE
    # 现在sp指向TrapContext， CSR_SAVE指向user stack

    # 保存ra返回地址
    SAVE_GP 1
    # 保存线程指针tp
    SAVE_GP 2

    # 保存通用寄存器
    # 这里跳过了r3，r3保存sp(栈指针)
    .set n, 4
    .rept 28
        SAVE_GP %n
        .set n, n+1
    .endr
    .set n, 0
=========================================
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    # 保存fscr寄存器
    movfcsr2gr $t0, $fcsr0
    st.w $t0, $sp, 84*8

    # save FCC
    movcf2gr $t0, $fcc7
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc6
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc5
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc4
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc3
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc2
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc1
    slli.w $t0, $t0, 1
    movcf2gr $t0, $fcc0
    # 保存 fcc寄存器
    st.b $t0, $sp, 82*8+4
==========================================
    # 保存sstatus寄存器
    csrrd $t0, CSR_PRMD
    st.d $t0, $sp, 32*8

    # 保存sepc
    csrrd $t1, CSR_ERA
    st.d $t1, $sp, 33*8

    # 将user stack地址读取到t2，然后保存在r3中(栈指针保存)
    csrrd $t2, CSR_SAVE
    st.d $t2, $sp, 3*8
    
    # 加载返回地址
    ld.d $ra, $sp, 35*8

    # 加载kernel寄存器(这些寄存器是callee saved)，需要手动加载
    # s0-s8 r12-r20
    ld.d $s0, $sp, 36*8
    ld.d $s1, $sp, 37*8
    ld.d $s2, $sp, 38*8
    ld.d $s3, $sp, 39*8
    ld.d $s4, $sp, 40*8
    ld.d $s5, $sp, 41*8
    ld.d $s6, $sp, 42*8
    ld.d $s7, $sp, 43*8
    ld.d $s8, $sp, 44*8
    ld.d $r12, $sp, 45*8
    ld.d $r13, $sp, 46*8
    ld.d $r14, $sp, 47*8
    ld.d $r15, $sp, 48*8
    ld.d $r16, $sp, 49*8
    ld.d $r17, $sp, 50*8
    ld.d $r18, $sp, 51*8
    ld.d $r19, $sp, 52*8
    ld.d $r20, $sp, 53*8
    
    # 加载fp
    ld.d $fp, $sp, 54*8
    # 加载tp
    ld.d $tp, $sp, 55*8
    # 加载kernel_sp
    ld.d $sp, $sp, 34*8

    # ret to uer_trap_handler
    ret


# kernel -> user
__return_to_user:
    # a0: *TrapContext in user space(Constant),

    move $sp, $a0
    csrwr  $a0, CSR_SAVE
    # now sp points to TrapContext in user space, start restoring based on it

=======================================-
    # restore FCSR
    ld.w $t0, $sp, 64*8
    movgr2fcsr $fcsr0, $t0


    # restore FCC
    ld.b $t0, $sp, 64*8+4
    movgr2cf $fcc0, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc1, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc2, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc3, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc4, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc5, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc6, $t0
    srli.w $t0, $t0, 1
    movgr2cf $fcc7, $t0
===========================================-

    # 恢复 CSR_PRMD/CSR_ERA
    ld.d $t0, $sp, 32*8
    ld.d $t1, $sp, 33*8
    csrwr $t0, CSR_PRMD
    csrwr $t1, CSR_ERA

    # 恢复用户寄存器，除了x0 和 sp
    LOAD_GP 1
    LOAD_GP 2
    .set n, 4
    .rept 28
        LOAD_GP %n
        .set n, n+1
    .endr

=====================================
    # 恢复浮点寄存器，后序要更改，只有在dirty才恢复
    .set n, 0
    .set m, FP_START
    .rept 32
        LOAD_FP %n, %m
        .set n, n+1
        .set m, m+1
    .endr
=======================================

    # 修改栈指针，返回用户栈
    ld.d $sp, $sp, 3*8
    ertn

.align 4
__trap_from_kernel:
    # 这里只需要保存caller saved寄存器
    addi.d  $sp, $sp, -264
    st.d    $ra, $sp, 0
    st.d    $tp, $sp, 8
    # st.d    $sp, $sp, 16
    st.d    $a0, $sp, 24
    st.d    $a1, $sp, 32
    st.d    $a2, $sp, 40
    st.d    $a3, $sp, 48
    st.d    $a4, $sp, 56
    st.d    $a5, $sp, 64
    st.d    $a6, $sp, 72
    st.d    $a7, $sp, 80
    st.d    $t0, $sp, 88
    st.d    $t1, $sp, 96
    st.d    $t2, $sp, 104
    st.d    $t3, $sp, 112
    st.d    $t4, $sp, 120
    st.d    $t5, $sp, 128
    st.d    $t6, $sp, 136
    st.d    $t7, $sp, 144
    st.d    $t8, $sp, 152
    st.d    $r21, $sp,160
    st.d    $fp, $sp, 168
    st.d    $s0, $sp, 176
    st.d    $s1, $sp, 184
    st.d    $s2, $sp, 192
    st.d    $s3, $sp, 200
    st.d    $s4, $sp, 208
    st.d    $s5, $sp, 216
    st.d    $s6, $sp, 224
    st.d    $s7, $sp, 232
    st.d    $s8, $sp, 240
    csrrd   $t0, 0x1        #读取prmd
    csrrd   $t1, 0x6        #返回地址
    st.d    $t0, $sp, 248
    st.d    $t1, $sp, 256

    la.abs  $t0, kernel_trap_handler
    # call the C trap handler in trap.c
    jirl    $ra, $t0, 0

    # restore register
    ld.d    $t0, $sp, 248
    ld.d    $t1, $sp, 256
    csrwr   $t0, 0x1
    csrwr   $t1, 0x6

    ld.d    $ra, $sp, 0
    ld.d    $tp, $sp, 8
    # ld.d    $sp, $sp, 16
    ld.d    $a0, $sp, 24
    ld.d    $a1, $sp, 32
    ld.d    $a2, $sp, 40
    ld.d    $a3, $sp, 48
    ld.d    $a4, $sp, 56
    ld.d    $a5, $sp, 64
    ld.d    $a6, $sp, 72
    ld.d    $a7, $sp, 80
    ld.d    $t0, $sp, 88
    ld.d    $t1, $sp, 96
    ld.d    $t2, $sp, 104
    ld.d    $t3, $sp, 112
    ld.d    $t4, $sp, 120
    ld.d    $t5, $sp, 128
    ld.d    $t6, $sp, 136
    ld.d    $t7, $sp, 144
    ld.d    $t8, $sp, 152
    ld.d    $r21, $sp,160
    ld.d    $fp, $sp, 168
    ld.d    $s0, $sp, 176
    ld.d    $s1, $sp, 184
    ld.d    $s2, $sp, 192
    ld.d    $s3, $sp, 200
    ld.d    $s4, $sp, 208
    ld.d    $s5, $sp, 216
    ld.d    $s6, $sp, 224
    ld.d    $s7, $sp, 232
    ld.d    $s8, $sp, 240

    addi.d  $sp, $sp, 264
    ertn

.section .text.sigret
.align 12
.global __sigret_helper
__sigret_helper:
    addi.d	$a7, $zero, 139
    syscall 0