use super::TaskControlBlock;
use alloc::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

pub struct ThreadGroup {
    pub tasks: BTreeMap<usize, Weak<TaskControlBlock>>,
}

impl ThreadGroup {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.tasks.insert(task.get_pid(), Arc::downgrade(&task));
    }

    pub fn remove(&mut self, taskid: usize) {
        self.tasks.remove(&taskid);
    }

    pub fn get(&self, taskid: usize) -> Option<&Weak<TaskControlBlock>> {
        self.tasks.get(&taskid)
    }

    #[allow(unused)]
    pub fn thread_num(&self) -> usize {
        self.tasks.len()
    }
}
