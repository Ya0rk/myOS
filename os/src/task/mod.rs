#[allow(clippy::module_inception)]
mod task;
mod manager;
mod pid;
mod processor;
mod fd;
mod sched;
mod thread_group;
pub mod aux;
pub mod executor;

pub use fd::{FdTable, FdInfo, sock_map_fd};
pub use pid::{KernelStack, Pid, PidAllocator};
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::CPU;
pub use sched::TaskFuture;
pub use pid::pid_alloc;
pub use manager::{
    add_task, remove_task_by_pid, get_task_by_pid, 
    remove_proc_group_member, add_proc_group_member, 
    new_process_group, get_init_proc, extract_proc_to_new_group,
    get_proc_num, get_target_proc_group, MANAGER
};
pub use sched::{spawn_user_task, spawn_kernel_task};
pub use processor::{
    init_processors, current_task, current_trap_cx, 
    current_user_token, take_current_task, 
    get_current_hart_id, get_current_cpu
};

use async_task::Task;
use log::info;
use crate::fs::flush_preload;
use crate::{fs::FileClass, sync::block_on};
use thread_group::ThreadGroup;
use crate::fs::OpenFlags;
use crate::fs::open_file;

///Add init process to the manager
pub async fn add_initproc() {
    let initproc = flush_preload().await;
    TaskControlBlock::new(initproc).await;
}
