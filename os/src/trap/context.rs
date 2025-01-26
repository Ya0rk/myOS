//! Implementation of [`TrapContext`]
use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct SstatusWrapper(Sstatus);

impl From<Sstatus> for SstatusWrapper {
    fn from(sstatus: Sstatus) -> Self {
        SstatusWrapper(sstatus)
    }
}

impl Into<Sstatus> for SstatusWrapper {
    fn into(self) -> Sstatus {
        self.0
    }
}

#[repr(C)]
pub struct TrapContext {
    /* 0-31 */ pub user_x: [usize; 32], 
    /*  32  */ pub sstatus: Sstatus,
    /*  33  */ pub sepc: usize,
    /*  34  */ pub kernel_sp: usize,
    /*  35  */ pub trap_loop: usize,
    /* 36-47*/ pub kernel_s: [usize; 12],
    /*  48  */ pub kernel_fp: usize,
    /*  49  */ pub kernel_tp: usize,
}

impl TrapContext {
    ///set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.user_x[2] = sp;
    }
    pub fn get_sp(&self) -> usize {
        self.user_x[2]
    }
    ///init app context
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_sp: usize,
        trap_loop: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            user_x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_sp,
            trap_loop,
            kernel_s: [0; 12],
            kernel_fp: 0,
            kernel_tp: 0,
        };
        cx.set_sp(sp);
        cx
    }
}
