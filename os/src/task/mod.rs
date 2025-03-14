mod context;
mod manager;
mod pid;
mod processor;
mod switch;
mod fd;
pub mod executor;
mod sched;
#[allow(clippy::module_inception)]
mod task;

pub use fd::{FdTable, Fd};
pub use manager::TaskManager;
pub use context::TaskContext;
pub use pid::{KernelStack, PidHandle, PidAllocator};
pub use task::{TaskControlBlock, TaskStatus};
pub use processor::Processor;
pub use sched::TaskFuture;
pub use pid::pid_alloc;
pub use manager::{add_task, remove_task_by_pid, get_task_by_pid};
pub use sched::{spawn_user_task, spawn_kernel_task};
pub use processor::{init_processors, current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task, get_current_hart_id};

use crate::arch::shutdown;
use crate::fs::FileClass;
use crate::fs::OpenFlags;
use alloc::sync::Arc;
use lazy_static::*;
use crate::fs::open_file;
use switch::__switch;

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    if let Some(task) = take_current_task(){
        // ---- access current TCB exclusively
        let task_cx_ptr = task.get_task_cx_mut() as *mut TaskContext;
        // Change status to Ready
        task.set_ready();
        // ---- release current PCB

        // push back to ready queue.
        add_task(&task);
        // jump to scheduling cycle
        schedule(task_cx_ptr);
    }
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // **** access current TCB exclusively
    // Change status to Zombie
    task.set_zombie();
    // Record exit code
    task.set_exit_code(exit_code);
    // do not move to its parent but under initproc

    // 将当前进程的子进程移动到initproc下
    // ++++++ access initproc TCB exclusively
    {
        for child in task.children.lock().iter() {
            child.set_parent(Some(Arc::downgrade(&INITPROC)));
            INITPROC.add_children(child.clone());
        }
    }
    // ++++++ release parent PCB

    // 删除当前进程的所有子进程
    task.clear_children();
    // deallocate user space
    task.recycle_data_pages();
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        // TODO: 重构为异步
        if let Some(FileClass::File(inode)) = open_file("initproc", OpenFlags::O_RDONLY) {
            let elf_data = inode.inode.read_all().unwrap();
            let res=TaskControlBlock::new(&elf_data);
            res
        } else {
            panic!("error: initproc from Abs File!");
        }
    };
}
///Add init process to the manager
pub fn add_initproc() {
    if let Some(FileClass::File(inode)) = open_file("initproc", OpenFlags::O_RDONLY) {
        let elf_data = inode.inode.read_all().unwrap();
        TaskControlBlock::new(&elf_data);
    } else {
        panic!("error: initproc from Abs File!");
    }
}
