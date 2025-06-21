pub mod aux;
pub mod executor;
mod fd;
pub mod futex;
mod ipc;
mod manager;
mod pid;
mod processor;
mod sched;
#[allow(clippy::module_inception)]
mod task;
mod thread_group;

pub use fd::{sock_map_fd, FdInfo, FdTable};
pub use futex::*;
pub use ipc::ShmidTable;
pub use manager::{
    add_proc_group_member, add_task, extract_proc_to_new_group, get_init_proc, get_proc_num,
    get_target_proc_group, get_task_by_pid, new_process_group, remove_proc_group_member,
    remove_task_by_pid, MANAGER,
};
pub use pid::pid_alloc;
pub use pid::{Pid, PidAllocator};
pub use processor::CPU;
pub use processor::{
    current_kernel_token, current_task, current_trap_cx, current_user_token, get_current_cpu,
    get_current_hart_id, init_processors, take_current_task,
    take_ktrap_ret, set_ktrap_ret
};
pub use sched::TaskFuture;
pub use sched::{spawn_kernel_task, spawn_user_task};
pub use task::{TaskControlBlock, TaskStatus};
pub use fd::test_fd_performance;

use crate::fs::{test_initproc, OpenFlags};
use crate::fs::{autorun, gbshell, initproc, mbshell};
use crate::{fs::FileClass, sync::block_on};
use async_task::Task;
use cfg_if::cfg_if;
use log::info;
use thread_group::ThreadGroup;

///Add init process to the manager
pub async fn add_initproc() {
    let file = if cfg!(feature = "autorun") {
        autorun().await
    } else if cfg!(feature = "gbshell") {
        gbshell().await
    } else if cfg!(feature = "mbshell") {
        mbshell().await
    } else if cfg!(feature = "initproc") {
        initproc().await
    } else if cfg!(feature = "test_initproc") {
        test_initproc().await
    } else {
        mbshell().await
    };

    TaskControlBlock::new(file).await;
}
