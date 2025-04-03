
use core::fmt::{Debug, Formatter};
use loongarch64::register::{CpuMode, prmd};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TrapContext {
    pub user_x: [usize; 32], //通用寄存器 ，第4个寄存器是sp
    pub prmd: usize,    //控制状态寄存器---似乎没有用
    pub sepc: usize,    //异常处理返回地址
    pub kernel_sp: usize,
    pub trap_handler: usize, //实际上就是trap_loop
    pub kernel_s: [usize; 12],
    pub kernel_fp: usize,
    pub kernel_tp: usize,
}

impl Debug for TrapContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TrapContext {{ x: {:?}, crmd: {:#b}, sepc: {:#x} }}",
            self.user_x, self.prmd, self.sepc
        )
    }
}


impl TrapContext {
    pub fn app_init_context(
        entry: usize, 
        sp: usize,
        kernel_sp: usize,
        trap_loop: usize,
    ) -> Self {
        // 设置为用户模式,trap使用ertn进入用户态时会被加载到crmd寄存器中
        prmd::set_pplv(CpuMode::Ring3);
        let mut cx = Self {
            user_x: [0; 32],
            prmd:prmd::read().raw(),
            sepc: entry,
            kernel_sp,
            trap_handler: trap_loop as usize,
            kernel_s: [0; 12],
            kernel_fp: 0,
            kernel_tp: 0,
        };
        cx.set_sp(sp);
        cx
    }
    pub fn set_sp(&mut self, sp: usize) {
        self.user_x[3] = sp;
    }
    pub fn get_sp(&mut self) -> usize {
        self.user_x[3]
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
}
