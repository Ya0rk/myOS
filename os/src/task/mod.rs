mod context;
mod manager;
mod pid;
mod processor;
mod switch;
mod fd;
pub mod executor;
mod sched;
mod thread_group;
#[allow(clippy::module_inception)]
mod task;

pub use fd::{FdTable, Fd};
pub use context::TaskContext;
pub use pid::{KernelStack, Pid, PidAllocator};
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::CPU;
pub use sched::TaskFuture;
pub use pid::pid_alloc;
pub use manager::{add_task, remove_task_by_pid, get_task_by_pid, remove_proc_group_member, add_proc_group_member, new_process_group};
pub use sched::{spawn_user_task, spawn_kernel_task};
pub use processor::{init_processors, current_task, current_trap_cx, current_user_token, take_current_task, get_current_hart_id};

use crate::fs::FileClass;
use thread_group::ThreadGroup;
use crate::fs::OpenFlags;
use alloc::sync::Arc;
use lazy_static::*;
use crate::fs::open_file;

// / Suspend the current 'Running' task and run the next task in task list.
// pub fn suspend_current_and_run_next() {
//     if let Some(task) = take_current_task(){
//         // ---- access current TCB exclusively
//         let task_cx_ptr = task.get_task_cx_mut() as *mut TaskContext;
//         // Change status to Ready
//         task.set_ready();
//         // ---- release current PCB

//         // push back to ready queue.
//         add_task(&task);
//         // jump to scheduling cycle
//         schedule(task_cx_ptr);
//     }
// }

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        // TODO: 重构为异步
        if let Some(FileClass::File(file)) = open_file("initproc", OpenFlags::O_RDONLY) {
            let elf_data = file.metadata.inode.read_all().unwrap();
            let res=TaskControlBlock::new(&elf_data);
            res
        } else {
            panic!("error: initproc from Abs File!");
        }
    };
}
///Add init process to the manager
pub fn add_initproc() {
    if let Some(FileClass::File(file)) = open_file("initproc", OpenFlags::O_RDONLY) {
        let elf_data = file.metadata.inode.read_all().unwrap();
        TaskControlBlock::new(&elf_data);
    } else {
        panic!("error: initproc from Abs File!");
    }
}
