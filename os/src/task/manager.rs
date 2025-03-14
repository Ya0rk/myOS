use super::TaskControlBlock;
use alloc::sync::{Arc, Weak};
use lazy_static::*;
use spin::Mutex;
use hashbrown::HashMap;

/// 存放所有任务的管理器，可以通过pid快速找到对应的Task
pub struct TaskManager {
    tasks: HashMap<usize, Weak<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
    /// 添加一个任务
    fn add(&mut self, task: &Arc<TaskControlBlock>) {
        let pid = task.getpid();
        self.tasks.insert(pid, Arc::downgrade(task));
    }
    
    /// 删除一个任务
    pub fn remove(&mut self, pid: usize) {
        self.tasks.remove(&pid);
    }

    /// 获取一个任务
    pub fn get(&self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        self.tasks.get(&pid).and_then(|weak| weak.upgrade())
    }
}

lazy_static! {
    // 承载有当前所有的任务
    // TODO: 使用RwLock能不能提高性能
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

/// 外露接口

///Interface offered to add task
pub fn add_task(task: &Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

///Interface offered to pop the first task
// pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
//     TASK_MANAGER.lock().fetch()
// }

/// 根据pid获取任务
pub fn get_task_by_pid(pid: usize) -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().get(pid)
}
/// 根据pid删除任务
pub fn remove_task_by_pid(pid: usize) {
    TASK_MANAGER.lock().remove(pid);
}