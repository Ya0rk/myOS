use core::arch::asm;
// use riscv::register::sstatus::FS;

use super::super::arch::sstatus::{self, Sstatus, SPP, FS};



#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserFloatRegs {
    f: [f64; 32], // 57-88
    fcsr: u32,    // 89
    fcc: u8,      // 89+4
    need_save: u8,
    need_restore: u8,
    dirty: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapContext {
    /* 0-31 */ pub user_x: [usize; 32], 
    /*  32  */ pub sstatus: PrMd,
    /*  33  */ pub sepc: usize,
    /*  34  */ pub kernel_sp: usize,
    /*  35  */ pub kernal_ra: usize,
    /* 36-53*/ pub kernel_s: [usize; 18], // 保存callee saved寄存器(s0-s8 r12-r20)
    /*  54  */ pub kernel_fp: usize,
    /*  55  */ pub kernel_tp: usize,
    /*  56  */ pub float_regs: UserFloatRegs,
}

impl TrapContext {
    ///init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_sp: usize,
        _trap_loop: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            user_x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_sp,
            kernal_ra: 0,
            kernel_s: [0; 12],
            kernel_fp: 0,
            kernel_tp: 0,
            float_regs: UserFloatRegs::new(),
        };
        cx.set_sp(sp);
        cx
    }
    pub fn set_sp(&mut self, sp: usize) {
        self.user_x[2] = sp;
    }
    pub fn get_sp(&self) -> usize {
        self.user_x[2]
    }
    pub fn set_tp(&mut self, tp: usize) {
        self.user_x[4] = tp;
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
    /// 在do_signal信号处理中,重新设置trap context
    /// 返回到用户自定义函数
    /// 
    /// handler: 信号处理 函数addr
    /// 
    /// new_sp: 信号处理栈的sp
    /// 
    /// sigret: 信号处理完后返回到sigreturn系统调用
    pub fn flash(&mut self, handler: usize, new_sp: usize, sigret: usize, signo: usize) {
        self.sepc = handler;
        self.set_sp(new_sp);
        self.user_x[1] = sigret;
        self.user_x[10] = signo;

    }
}

impl UserFloatRegs {
    fn new() -> Self {
        Self {
            f: [0.0; 32],
            fcsr: 0,
            need_save: 0,
            need_restore: 0,
            dirty: 0,
        }
    }

    /// 在任务切换到内核态时，判断是否需要保存浮点寄存器的内容
    pub fn trap_in_do_with_freg(&mut self, sstatus: Sstatus) {
        if sstatus.fs() == FS::Dirty {
            self.need_save = 1;
        }
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

    // TODO: 完善信号处理时 是否需要保存浮点寄存器的内容
    #[cfg(target_arch = "riscv64")]
    pub fn save(&mut self) {
        if self.need_save == 0 {
            return;
        }
        self.need_save = 0;
        unsafe {
            let mut _t: usize = 1; // alloc a register but not zero.
            asm!("
            fsd  f0,  0*8({0})
            fsd  f1,  1*8({0})
            fsd  f2,  2*8({0})
            fsd  f3,  3*8({0})
            fsd  f4,  4*8({0})
            fsd  f5,  5*8({0})
            fsd  f6,  6*8({0})
            fsd  f7,  7*8({0})
            fsd  f8,  8*8({0})
            fsd  f9,  9*8({0})
            fsd f10, 10*8({0})
            fsd f11, 11*8({0})
            fsd f12, 12*8({0})
            fsd f13, 13*8({0})
            fsd f14, 14*8({0})
            fsd f15, 15*8({0})
            fsd f16, 16*8({0})
            fsd f17, 17*8({0})
            fsd f18, 18*8({0})
            fsd f19, 19*8({0})
            fsd f20, 20*8({0})
            fsd f21, 21*8({0})
            fsd f22, 22*8({0})
            fsd f23, 23*8({0})
            fsd f24, 24*8({0})
            fsd f25, 25*8({0})
            fsd f26, 26*8({0})
            fsd f27, 27*8({0})
            fsd f28, 28*8({0})
            fsd f29, 29*8({0})
            fsd f30, 30*8({0})
            fsd f31, 31*8({0})
            csrr {1}, fcsr
            sw  {1}, 32*8({0})
        ", in(reg) self,
                inout(reg) _t
            );
        };
    }
    #[cfg(target_arch = "loongarch64")]
    pub fn save(&mut self) {

    }

    /// Restore mem -> reg
    #[cfg(target_arch = "riscv64")]
    pub fn restore(&mut self) {
        if self.need_restore == 0 {
            return;
        }
        self.need_restore = 0;
        unsafe {
            asm!("
            fld  f0,  0*8({0})
            fld  f1,  1*8({0})
            fld  f2,  2*8({0})
            fld  f3,  3*8({0})
            fld  f4,  4*8({0})
            fld  f5,  5*8({0})
            fld  f6,  6*8({0})
            fld  f7,  7*8({0})
            fld  f8,  8*8({0})
            fld  f9,  9*8({0})
            fld f10, 10*8({0})
            fld f11, 11*8({0})
            fld f12, 12*8({0})
            fld f13, 13*8({0})
            fld f14, 14*8({0})
            fld f15, 15*8({0})
            fld f16, 16*8({0})
            fld f17, 17*8({0})
            fld f18, 18*8({0})
            fld f19, 19*8({0})
            fld f20, 20*8({0})
            fld f21, 21*8({0})
            fld f22, 22*8({0})
            fld f23, 23*8({0})
            fld f24, 24*8({0})
            fld f25, 25*8({0})
            fld f26, 26*8({0})
            fld f27, 27*8({0})
            fld f28, 28*8({0})
            fld f29, 29*8({0})
            fld f30, 30*8({0})
            fld f31, 31*8({0})
            lw  {0}, 32*8({0})
            csrw fcsr, {0}
        ", in(reg) self
            );
        }
    }
    #[cfg(target_arch = "loongarch64")]
    pub fn restore(&mut self) {
        unimplemented!()
    }
}