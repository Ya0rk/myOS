// mod context;
mod manager;
mod pid;
mod processor;
// mod switch;
mod fd;
pub mod executor;
mod sched;
mod thread_group;
#[allow(clippy::module_inception)]
mod task;
pub mod aux;

pub use fd::{FdTable, Fd};
// pub use context::TaskContext;
pub use pid::{KernelStack, Pid, PidAllocator};
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::CPU;
pub use sched::TaskFuture;
pub use pid::pid_alloc;
pub use manager::{
    add_task, remove_task_by_pid, get_task_by_pid, 
    remove_proc_group_member, add_proc_group_member, 
    new_process_group, get_init_proc, extract_proc_to_new_group,
    get_proc_num
};
pub use sched::{spawn_user_task, spawn_kernel_task};
pub use processor::{
    init_processors, current_task, current_trap_cx, 
    current_user_token, take_current_task, 
    get_current_hart_id
};

use crate::{fs::FileClass, sync::block_on};
use thread_group::ThreadGroup;
use crate::fs::OpenFlags;
use crate::fs::open_file;

// lazy_static! {
//     ///Globle process that init user shell
//     pub static ref INITPROC: Arc<TaskControlBlock> = {
//         // TODO: 重构为异步
//         if let Some(FileClass::File(file)) = open_file("initproc", OpenFlags::O_RDONLY) {
//             let elf_data = file.metadata.inode.read_all().await.unwrap();
//             let res=TaskControlBlock::new(&elf_data);
//             res
//         } else {
//             panic!("error: initproc from Abs File!");
//         }
//     };
// }
///Add init process to the manager
pub fn add_initproc() {
    if let Some(file) = open_file("initproc", OpenFlags::O_RDONLY) {
        // let elf_data = block_on(async { file.metadata.inode.read_all().await }).unwrap();
        TaskControlBlock::new(file);
    } else {
        panic!("error: initproc from Abs File!");
    }
}
