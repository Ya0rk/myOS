use crate::trap::trap_loop;

#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /* 0 */  ra: usize,
    /* 1 */  sp: usize,
    /*2-13*/ s: [usize; 12], // s0-11 register, callee saved
}

impl TaskContext {
    /// init task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_trap_loop(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_loop as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
