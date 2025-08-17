use super::super::arch::sstatus::{self, Sstatus, FS, SPP};
use core::{arch::asm, fmt::Debug};
use log::info;
use loongarch64::register::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UserFloatRegs {
    pub f: [f64; 32], // 56-87
    pub fcsr: u32,    // 88
    pub fcc: u8,      // 88+4
    pub need_save: u8,
    pub need_restore: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TrapContext {
    /// 通用寄存器
    /* 0-31 */
    pub user_gp: GPRegs,
    /*  32  */ pub sstatus: Sstatus,
    /*  33  */ pub sepc: usize,
    /*  34  */ pub kernel_sp: usize,
    /*  35  */ pub kernal_ra: usize,
    /* 36-53*/ pub kernel_s: [usize; 18], // 保存callee saved寄存器(s0-s8 r12-r20)
    /*  54  */ pub kernel_fp: usize,
    /*  55  */ pub kernel_tp: usize,
    /*  56  */ pub float_regs: UserFloatRegs,
}

/// 通用寄存器
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct GPRegs {
    /// r0 - 硬连线为常数0的寄存器（zero）
    pub zero: usize,
    /// r1 - 返回地址寄存器（ra）
    pub ra: usize,
    /// r2 - 线程局部存储指针（tp）
    pub tp: usize,
    /// r3 - 栈指针寄存器（sp）
    pub sp: usize,
    /// r4 - 参数/返回值寄存器0（a0/v0）
    pub a0: usize,
    /// r5 - 参数/返回值寄存器1（a1/v1）
    pub a1: usize,
    /// r6 - 参数寄存器2（a2）
    pub a2: usize,
    /// r7 - 参数寄存器3（a3）
    pub a3: usize,
    /// r8 - 参数寄存器4（a4）
    pub a4: usize,
    /// r9 - 参数寄存器5（a5）
    pub a5: usize,
    /// r10 - 参数寄存器6（a6）
    pub a6: usize,
    /// r11 - 参数寄存器7（a7）
    pub a7: usize,
    /// r12 - 临时寄存器0（t0）
    pub t0: usize,
    /// r13 - 临时寄存器1（t1）
    pub t1: usize,
    /// r14 - 临时寄存器2（t2）
    pub t2: usize,
    /// r15 - 临时寄存器3（t3）
    pub t3: usize,
    /// r16 - 临时寄存器4（t4）
    pub t4: usize,
    /// r17 - 临时寄存器5（t5）
    pub t5: usize,
    /// r18 - 临时寄存器6（t6）
    pub t6: usize,
    /// r19 - 临时寄存器7（t7）
    pub t7: usize,
    /// r20 - 临时寄存器8（t8）
    pub t8: usize,
    /// r21 - 保留寄存器（未分配用途）
    pub r21: usize,
    /// r22 - 帧指针寄存器（fp/s9）
    pub fp: usize,
    /// r23 - 静态寄存器0（s0）
    pub s0: usize,
    /// r24 - 静态寄存器1（s1）
    pub s1: usize,
    /// r25 - 静态寄存器2（s2）
    pub s2: usize,
    /// r26 - 静态寄存器3（s3）
    pub s3: usize,
    /// r27 - 静态寄存器4（s4）
    pub s4: usize,
    /// r28 - 静态寄存器5（s5）
    pub s5: usize,
    /// r29 - 静态寄存器6（s6）
    pub s6: usize,
    /// r30 - 静态寄存器7（s7）
    pub s7: usize,
    /// r31 - 静态寄存器8（s8）
    pub s8: usize,
}


impl GPRegs {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn as_slice(&self) -> &[usize; 32] {
        unsafe { core::mem::transmute(self) }
        // core::slice::from_raw_parts(self as *const usize, 32)
    }
    pub fn as_mut_slice(&mut self) -> &mut [usize; 32] {
        unsafe { core::mem::transmute(self) }
    }
}

impl TrapContext {
    ///init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        //kernel_sp: usize,
        //_trap_loop: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            user_gp: GPRegs::new(),
            sstatus,
            sepc: entry,
            kernel_sp: 0,
            kernal_ra: 0,
            kernel_s: [0; 18],
            kernel_fp: 0,
            kernel_tp: 0,
            float_regs: UserFloatRegs::new(),
        };
        cx.set_sp(sp);
        cx
    }
    pub fn gp_regs(&mut self) -> &mut [usize; 32] {
        self.user_gp.as_mut_slice()
    }

    /// 设置context参数
    pub fn set_arg(&mut self, argc: usize, argv: usize, env: usize) {
        self.user_gp.a0 = argc;
        self.user_gp.a1 = argv;
        self.user_gp.a2 = env;
        self.float_regs = UserFloatRegs::new();
    }
    pub fn set_sp(&mut self, sp: usize) {
        self.user_gp.sp = sp;
    }
    pub fn get_sp(&self) -> usize {
        self.user_gp.sp
    }
    pub fn set_tp(&mut self, tp: usize) {
        self.user_gp.tp = tp;
    }
    pub fn set_sepc(&mut self, sepc: usize) {
        self.sepc = sepc;
    }
    pub fn get_sepc(&self) -> usize {
        self.sepc
    }
    pub fn set_kernel_sp(&mut self, kernel_sp: usize) {
        self.kernel_sp = kernel_sp;
    }
    pub fn get_fp(&self) -> usize {
        self.user_gp.fp
    }
    /// 在do_signal信号处理中,重新设置trap context
    /// 返回到用户自定义函数
    ///
    /// handler: 信号处理 函数addr
    ///
    /// new_sp: 信号处理栈的sp
    ///
    /// sigret: 信号处理完后返回到sigreturn系统调用
    pub fn flash(
        &mut self,
        handler: usize,
        new_sp: usize,
        sigret: usize,
        signo: usize,
        gp: usize,
        tp: usize,
    ) {
        info!(
            "[flash] in with args: handler:{:#x}, new_sp:{:#x}, sigret:{:#x}, signo:{}",
            handler, new_sp, sigret, signo
        );
        self.sepc = handler;
        self.set_sp(new_sp);
        self.user_gp.ra = (sigret & 0xFFFF_FFFF_FFFF) | 0xFFFF_FFC0_0000_0000;
        self.user_gp.a0 = signo;
        self.user_gp.tp = tp;
    }
}

impl UserFloatRegs {
    fn new() -> Self {
        Self {
            f: [0.0; 32],
            fcsr: 0,
            fcc: 0,
            need_save: 0,
            need_restore: 0,
        }
    }

    /// 在任务切换到内核态时，判断是否需要保存浮点寄存器的内容
    pub fn trap_in_do_with_freg(&mut self, sstatus: Sstatus) {
        // if sstatus.fs() == FS::Dirty {
        //     self.need_save = 1;
        // }
    }

    /// 在内核态切换到任务时，恢复浮点寄存器的内容
    pub fn trap_out_do_with_freg(&mut self) {
        self.restore();
    }

    /// 在任务调度时，将浮点寄存器的内容保存到内存中
    pub fn sched_out_do_with_freg(&mut self) {
        if self.need_save == 0 {
            return;
        }
        self.save();
        self.need_restore = 1;
    }

    #[cfg(target_arch = "loongarch64")]
    pub fn save(&mut self) {
        // TODO(YJJ)：后续实现检测dirty然后保存和恢复等操作
        return;
    }

    #[cfg(target_arch = "loongarch64")]
    pub fn restore(&mut self) {
        // TODO(YJJ)：后续实现检测dirty然后保存和恢复等操作
        return;
    }
}
