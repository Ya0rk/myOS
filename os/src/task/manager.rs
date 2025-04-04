use crate::{config::INITPROC_PID, sync::SpinNoIrqLock};
use super::TaskControlBlock;
use alloc::{sync::{Arc, Weak}, vec::Vec};
use lazy_static::*;
use hashbrown::HashMap;

type Pid = usize;
type PGid = usize;

lazy_static! {
    pub static ref MANAGER: Manager = Manager::new();
}

pub struct Manager {
    pub task_manager: SpinNoIrqLock<TaskManager>,
    pub process_group: SpinNoIrqLock<ProcessGroupManager>,
}

/// 存放所有任务的管理器，可以通过pid快速找到对应的Task
pub struct TaskManager(HashMap<Pid, Weak<TaskControlBlock>>);
/// 存放进程组的管理器，通过进程组的leader 的pid可以定位到进程组
pub struct ProcessGroupManager(HashMap<PGid, Vec<Pid>>);

impl Manager {
    pub fn new() -> Self {
        Self { 
            task_manager: SpinNoIrqLock::new(TaskManager(HashMap::new())),
            process_group: SpinNoIrqLock::new(ProcessGroupManager(HashMap::new())),
        }
    }
}

/// A simple FIFO scheduler.
impl TaskManager {
    /// 添加一个任务
    fn add(&mut self, task: &Arc<TaskControlBlock>) {
        let pid = task.get_pid();
        self.0.insert(pid, Arc::downgrade(task));
    }
    
    /// 删除一个任务
    pub fn remove(&mut self, pid: Pid) {
        self.0.remove(&pid);
    }

    /// 获取一个任务
    pub fn get(&self, pid: Pid) -> Option<Arc<TaskControlBlock>> {
        self.0.get(&pid).and_then(|weak| weak.upgrade())
    }
}

/// 添加任务
pub fn add_task(task: &Arc<TaskControlBlock>) {
    MANAGER.task_manager.lock().add(task);
}
/// 根据pid获取任务
pub fn get_task_by_pid(pid: usize) -> Option<Arc<TaskControlBlock>> {
    MANAGER.task_manager.lock().get(pid)
}
/// 根据pid删除任务
pub fn remove_task_by_pid(pid: usize) {
    MANAGER.task_manager.lock().remove(pid);
}
/// 获取到init proc
pub fn get_init_proc() -> Arc<TaskControlBlock> {
    MANAGER.task_manager.lock().get(INITPROC_PID).unwrap()
}

impl ProcessGroupManager {
    fn add_new_group(&mut self, pgid: PGid) {
        let mut vec: Vec<usize> = Vec::new();
        if pgid != INITPROC_PID {
            vec.push(pgid);
        }
        self.0.insert(pgid, vec);
    }

    fn add(&mut self, pgid: PGid, pid: Pid) {
        let target_group = self.0.get_mut(&pgid).unwrap();
        target_group.push(pid);
    }

    fn remove(&mut self, pgid: PGid, pid: Pid) {
        let target_group = self.0.get_mut(&pgid).unwrap();
        target_group.remove(pid);
    }
}

pub fn new_process_group(pgid: PGid) {
    MANAGER.process_group.lock().add_new_group(pgid);
}

pub fn remove_proc_group_member(pgid: PGid, pid: Pid) {
    MANAGER.process_group.lock().remove(pgid, pid);
}

pub fn add_proc_group_member(pgid: PGid, pid: Pid) {
    MANAGER.process_group.lock().add(pgid, pid);
}