use super::{Fd, FdTable, TaskContext};
use super::{pid_alloc, KernelStack, PidHandle};
use crate::config::USER_TRAP_CONTEXT;
use crate::mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr};
use crate::trap::{trap_loop, TrapContext};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use log::info;
use spin::{Mutex, MutexGuard};

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: Mutex<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    // pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub fd_table: FdTable,
    pub current_path: String,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn get_current_path(&self) -> String {
        self.current_path.clone()
    }
    pub fn fd_table_len(&self) -> usize {
        self.fd_table.table_len()
    }
    /// 判断是否打开文件描述符fd
    pub fn fd_is_none(&self, fd: usize) -> bool {
        self.fd_table.table[fd].is_none()
    }
    /// 将fd作为index获取文件描述符
    pub fn get_fd(&self, fd: usize) -> Fd {
        self.fd_table.get_fd(fd).unwrap()
    }
    
}

impl TaskControlBlock {
    pub fn inner_lock(&self) -> MutexGuard<'_, TaskControlBlockInner> {
        self.inner.lock()
    }
    pub fn get_ppid(&self) -> usize {
        self.inner_lock().parent.as_ref().map(|p| p.upgrade().unwrap().pid.0).unwrap_or(0)
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    /// 创建新task,只有initproc会调用
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(USER_TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        
        // 每个task有自己的kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack.get_kernel_stack_pos();
        memory_set.insert_framed_area(
            kernel_stack_bottom.into(), 
            kernel_stack_top.into(), 
            MapPermission::R | MapPermission::W,
        );
        
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: 
                Mutex::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_loop(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: FdTable::new(),
                    current_path: String::from("/"), // root directory
                }),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_lock().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            kernel_stack_top,
            trap_loop as usize,
        );
        println!("initproc successfully created, pid: {}", task_control_block.getpid());
        println!("initproc entry: {:#x}, sp: {:#x}", entry_point, user_sp);
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        info!("exec start");
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(USER_TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        
        // 建立该进程的kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = self.kernel_stack.get_kernel_stack_pos();
        memory_set.insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        info!("exec memory_set created");
        
        // **** access inner exclusively
        let mut inner = self.inner_lock();
        info!("satp before : {:#x}", inner.memory_set.token());
        // substitute memory_set
        inner.memory_set = memory_set;
        info!("satp after : {:#x}", inner.memory_set.token());
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;

        // initialize trap_cx
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            self.kernel_stack.get_top(),
            trap_loop as usize,
        );
        *inner.get_trap_cx() = trap_cx;
        info!("task.exec.pid={}", self.pid.0);
        // **** release inner automatically
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_lock();
        // copy user space(include trap context)
        let mut child_memory_set = MemorySet::clone_from_existed_proc(&parent_inner.memory_set);
        let child_trap_cx_ppn = child_memory_set
            .translate(VirtAddr::from(USER_TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack.get_kernel_stack_pos();
        child_memory_set.insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner:
                Mutex::new(TaskControlBlockInner {
                    trap_cx_ppn: child_trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_loop(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set: child_memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: parent_inner.fd_table.clone(),    // copy fd table
                    current_path: parent_inner.current_path.clone(),
                }),
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_lock().get_trap_cx();
        trap_cx.set_kernel_sp(kernel_stack_top);
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
