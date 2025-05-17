
FP_START = 56
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

# .section .text
#     .globl __trap_from_user
#     .globl __return_to_user
#     .global __trap_from_kernel
#     .align 4

.equ CSR_SAVE, 0x30
# 保存sepc
.equ CSR_ERA, 0x6
# sstatus：
.equ CSR_PRMD, 0x1
.equ CSR_PGDL, 0x19
.equ CSR_PGD, 0x1b

# user -> kernel
.section .text.usertrap
    .globl __trap_from_user
    .globl __return_to_user
    .align 4

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

    # 保存浮点数寄存器(TODO(YJJ):后续可以优化，判断为dirty才保存和恢复)
    .set n, 0
    .set m, FP_START
    .rept 32
        SAVE_FP %n, %m
        .set n, n+1
        .set m, m+1
    .endr

    # 保存fscr寄存器
    movfcsr2gr $t0, $fcsr0
    st.w $t0, $sp, 89*8

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
    st.b $t0, $sp, 89*8+4

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

    # 将a0与CSR_SAVE交换，其实是想写入CSR_SAVE
    csrwr  $a0, CSR_SAVE
    # 将a0赋值给sp，现在sp指向TrapContext
    csrwr  $sp, CSR_SAVE
# __trap_breakpoint:
    # now sp points to TrapContext in user space, start restoring based on it

    # 保存kernel寄存器(这些寄存器是callee saved)
    # s0-s8 r12-r20
    st.d $s0, $sp, 36*8
    st.d $s1, $sp, 37*8
    st.d $s2, $sp, 38*8
    st.d $s3, $sp, 39*8
    st.d $s4, $sp, 40*8
    st.d $s5, $sp, 41*8
    st.d $s6, $sp, 42*8
    st.d $s7, $sp, 43*8
    st.d $s8, $sp, 44*8
    st.d $r12, $sp, 45*8
    st.d $r13, $sp, 46*8
    st.d $r14, $sp, 47*8
    st.d $r15, $sp, 48*8
    st.d $r16, $sp, 49*8
    st.d $r17, $sp, 50*8
    st.d $r18, $sp, 51*8
    st.d $r19, $sp, 52*8
    st.d $r20, $sp, 53*8

    # 保存sp
    csrrd $a0, CSR_SAVE
    st.d $a0, $sp, 34*8

    # 保存ra
    st.d $ra, $sp, 35*8
    # 保存fp
    st.d $fp, $sp, 54*8
    # 保存线程指针tp
    st.d $tp, $sp, 55*8

    # 恢复 FCSR
    ld.w $t0, $sp, 89*8
    movgr2fcsr $fcsr0, $t0

    # 恢复 FCC
    ld.b $t0, $sp, 89*8+4
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

    # 恢复浮点寄存器，(TODO:YJJ)后序要更改，只有在dirty才恢复
    .set n, 0
    .set m, FP_START
    .rept 32
        LOAD_FP %n, %m
        .set n, n+1
        .set m, m+1
    .endr

    csrwr $sp, CSR_SAVE
    csrrd $sp, CSR_SAVE
    # 修改栈指针，返回用户栈
    ld.d $sp, $sp, 3*8
    ertn

    .section .text.kerneltrap
    .globl __trap_from_kernel
    .align 4
__trap_from_kernel:
    # 这里只需要保存caller saved寄存器
    addi.d  $sp, $sp, -34*8
    st.d    $ra, $sp, 1*8
    st.d    $tp, $sp, 2*8
    st.d    $a0, $sp, 4*8
    st.d    $a1, $sp, 5*8
    st.d    $a2, $sp, 6*8
    st.d    $a3, $sp, 7*8
    st.d    $a4, $sp, 9*8
    st.d    $a5, $sp, 10*8
    st.d    $a6, $sp, 11*8
    st.d    $a7, $sp, 12*8
    st.d    $t0, $sp, 13*8
    st.d    $t1, $sp, 14*8
    st.d    $t2, $sp, 15*8
    st.d    $t3, $sp, 16*8
    st.d    $t4, $sp, 17*8
    st.d    $t5, $sp, 18*8
    st.d    $t6, $sp, 19*8
    st.d    $t7, $sp, 20*8
    st.d    $t8, $sp, 21*8
    st.d    $r21, $sp,22*8
    # st.d    $fp, $sp, 168
    st.d    $s0, $sp, 23*8
    st.d    $s1, $sp, 24*8
    st.d    $s2, $sp, 25*8
    st.d    $s3, $sp, 26*8
    st.d    $s4, $sp, 27*8
    st.d    $s5, $sp, 28*8
    st.d    $s6, $sp, 29*8
    st.d    $s7, $sp, 30*8
    st.d    $s8, $sp, 31*8

    csrrd   $t0, 0x1        #读取prmd
    st.d    $t0, $sp, 32*8

    csrrd   $t1, 0x6        #返回地址
    st.d    $t1, $sp, 33*8

    # 跳转到kernel_trap_handler
    la.abs  $t0, kernel_trap_handler
    jirl    $ra, $t0, 0

    # 恢复prmd
    ld.d    $t0, $sp, 32*8
    csrwr   $t0, 0x1

    ld.d    $t1, $sp, 33*8
    csrwr   $t1, 0x6

    ld.d    $ra, $sp, 1*8
    ld.d    $tp, $sp, 2*8
    ld.d    $a0, $sp, 4*8
    ld.d    $a1, $sp, 5*8
    ld.d    $a2, $sp, 6*8
    ld.d    $a3, $sp, 7*8
    ld.d    $a4, $sp, 9*8
    ld.d    $a5, $sp, 10*8
    ld.d    $a6, $sp, 11*8
    ld.d    $a7, $sp, 12*8
    ld.d    $t0, $sp, 13*8
    ld.d    $t1, $sp, 14*8
    ld.d    $t2, $sp, 15*8
    ld.d    $t3, $sp, 16*8
    ld.d    $t4, $sp, 17*8
    ld.d    $t5, $sp, 18*8
    ld.d    $t6, $sp, 19*8
    ld.d    $t7, $sp, 20*8
    ld.d    $t8, $sp, 21*8
    ld.d    $r21, $sp,22*8
    ld.d    $s0, $sp, 23*8
    ld.d    $s1, $sp, 24*8
    ld.d    $s2, $sp, 25*8
    ld.d    $s3, $sp, 26*8
    ld.d    $s4, $sp, 27*8
    ld.d    $s5, $sp, 28*8
    ld.d    $s6, $sp, 29*8
    ld.d    $s7, $sp, 30*8
    ld.d    $s8, $sp, 31*8

    addi.d  $sp, $sp, 34*8
    ertn

.section .text.sigret
.align 12
.global __sigret_helper
__sigret_helper:
    addi.d	$a7, $zero, 139
    syscall 0