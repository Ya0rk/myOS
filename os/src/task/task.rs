use core::cell::SyncUnsafeCell;
use core::sync::atomic::AtomicI32;
use core::task::Waker;

use super::{Fd, FdTable, TaskContext};
use super::{pid_alloc, KernelStack, PidHandle};
use crate::arch::shutdown;
use crate::fs::File;
use crate::mm::{MapPermission, MemorySet};
use crate::sync::{new_shared, Shared, TimeData};
use crate::task::{spawn_user_task, INITPROC};
use crate::trap::{trap_loop, TrapContext};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use log::{debug, info};

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

pub struct TaskControlBlock {
    // 不可变
    pid:            PidHandle,
    kernel_stack:   KernelStack,

    // 可变
    // inner: SpinNoIrqLock<TaskControlBlockInner>,
    
    base_size:      Shared<usize>,
    /// 进程状态: Ready, Running, Zombie
    task_status:    Shared<TaskStatus>,
    memory_set:     Shared<MemorySet>,
    parent:         Shared<Option<Weak<TaskControlBlock>>>,
    pub children:   Shared<Vec<Arc<TaskControlBlock>>>,
    fd_table:       Shared<FdTable>,
    current_path:   Shared<String>,

    waker:          SyncUnsafeCell<Option<Waker>>,
    trap_cx:        SyncUnsafeCell<TrapContext>,
    task_cx:        SyncUnsafeCell<TaskContext>, // 会删除
    time_data:      SyncUnsafeCell<TimeData>,

    exit_code:      AtomicI32,
}

impl TaskControlBlock {
    pub fn get_ppid(&self) -> usize {
        self.parent.lock().as_ref().map(|p| p.upgrade().unwrap().pid.0).unwrap_or(0)
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    /// 创建新task,只有initproc会调用
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            0,
            trap_loop as usize,
        );
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
        let task_control_block = Arc::new(Self {
            pid: pid_handle,
            kernel_stack,
            
            // Shared
            base_size: new_shared(user_sp),
            task_status: new_shared(TaskStatus::Ready),
            memory_set: new_shared(memory_set),
            parent: new_shared(None),
            children: new_shared(Vec::new()),
            fd_table: new_shared(FdTable::new()),
            current_path: new_shared(String::from("/")), // root directory
            
            // SyncUnsafeCell
            waker:   SyncUnsafeCell::new(None),
            trap_cx: SyncUnsafeCell::new(trap_cx),
            task_cx: SyncUnsafeCell::new(TaskContext::goto_trap_loop(kernel_stack_top)),
            time_data: SyncUnsafeCell::new(TimeData::new()),

            exit_code: AtomicI32::new(0),
        });

        debug!("initproc successfully created, pid: {}", task_control_block.getpid());
        debug!("initproc entry: {:#x}, sp: {:#x}", entry_point, user_sp);

        spawn_user_task(task_control_block.clone());

        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        debug!("exec start");
        let (mut memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        
        // 建立该进程的kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = self.kernel_stack.get_kernel_stack_pos();
        memory_set.insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        debug!("exec memory_set created");
        
        // **** access inner exclusively
        // substitute memory_set
        let mut mem = self.memory_set.lock();
        *mem = memory_set;

        // initialize trap_cx
        let trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            self.kernel_stack.get_top(),
            trap_loop as usize,
        );
        let old_trap_cx = self.get_trap_cx_mut();
        *old_trap_cx = trap_cx;
        
        debug!("task.exec.pid={}", self.pid.0);
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let parent = self;
        // copy user space(include trap context)
        let mut child_memory_set = MemorySet::clone_from_existed_proc(&parent.memory_set.lock());

        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack.get_kernel_stack_pos();

        // modify kernel_sp in trap_cx
        let trap_cx = parent.get_trap_cx_mut();
        trap_cx.set_kernel_sp(kernel_stack_top);

        child_memory_set.insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,

            // Shared
            base_size: parent.base_size.clone(),
            task_status: new_shared(TaskStatus::Ready),
            memory_set: new_shared(child_memory_set),
            parent: new_shared(Some(Arc::downgrade(self))),
            children: new_shared(Vec::new()),
            fd_table: parent.fd_table.clone(),    // copy fd table
            current_path: parent.current_path.clone(),

            // SyncUnsafeCell
            waker  : SyncUnsafeCell::new(None),
            trap_cx: SyncUnsafeCell::new(*trap_cx),
            task_cx: SyncUnsafeCell::new(TaskContext::goto_trap_loop(kernel_stack_top)),
            time_data: SyncUnsafeCell::new(TimeData::new()),

            exit_code: AtomicI32::new(0),
        });
        // add child
        parent.children.lock().push(task_control_block.clone());
        
        task_control_block
    }
    
    pub fn exit(&self) {
        info!("Task {} exit;", self.getpid());
        let pid = self.getpid();

        // 如果是idle进程
        if pid == 0 {
            info!("Idle process exit with exit_code {} ...", self.get_exit_code());
            shutdown(false);
        }

        // 将当前进程的子进程移动到initproc下
        {
            for child in self.children.lock().iter() {
                child.set_parent(Some(Arc::downgrade(&INITPROC)));
                INITPROC.add_children(child.clone());
            }
        }

        self.clear_children();
        self.clear_fd_table();
        self.recycle_data_pages();
        self.set_zombie();
    }
}

impl TaskControlBlock {
    /// 获取time_data
    pub fn get_time_data(&self) -> &TimeData {
        unsafe { &*self.time_data.get() }
    }
    pub fn get_time_data_mut(&self) -> &mut TimeData {
        unsafe { &mut *(self.time_data.get() as *mut TimeData) }
    }

    /// 获取当前进程的pid
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.lock().token()
    }

    /// 进程状态
    pub fn get_status(&self) -> TaskStatus {
        *self.task_status.lock()
    }
    pub fn set_ready(&self) {
        *self.task_status.lock() = TaskStatus::Ready;
    }
    pub fn set_running(&self) {
        *self.task_status.lock() = TaskStatus::Running;
    }
    pub fn set_zombie(&self) {
        *self.task_status.lock() = TaskStatus::Zombie;
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }

    /// task context
    pub fn get_task_cx(&self) -> &TaskContext {
        unsafe { &*self.task_cx.get() }
    }
    pub fn get_task_cx_mut(&self) -> &'static mut TaskContext {
        unsafe { &mut *(self.task_cx.get() as *mut TaskContext) }
    }
    pub fn get_trap_cx(&self) -> &TrapContext {
        unsafe { &*self.trap_cx.get() }
    }
    pub fn get_trap_cx_mut(&self) -> &'static mut TrapContext {
        unsafe { &mut *(self.trap_cx.get() as *mut TrapContext) }
    }

    /// 刷新TLB
    pub fn switch_pgtable(&self) {
        unsafe { self.memory_set.lock().activate() };
    }

    // exit code
    /// 获取进程的exit code
    pub fn get_exit_code(&self) -> i32 {
        self.exit_code.load(core::sync::atomic::Ordering::Relaxed)
    }
    /// 修改进程exit code
    pub fn set_exit_code(&self, exit_code: i32) {
        self.exit_code.store(exit_code, core::sync::atomic::Ordering::Relaxed);
    }

    // children
    // /// 获取所有子进程： Vec
    // pub fn children(&self) -> Vec<Arc<TaskControlBlock>> {
    //     self.children.lock().clone()
    // }
    /// 添加子进程
    pub fn add_children(&self, child: Arc<TaskControlBlock>) {
        self.children.lock().push(child);
    }
    /// 移除所有子进程
    pub fn clear_children(&self) {
        self.children.lock().clear();
    }
    /// 设置父进程
    pub fn set_parent(&self, parent: Option<Weak<TaskControlBlock>>) {
        *self.parent.lock() = parent;
    }

    /// 移除所有的 `MapArea`
    pub fn recycle_data_pages(&self) {
        self.memory_set.lock().recycle_data_pages();
    }
    
    // fd
    /// 通过fd获取文件
    pub fn get_file_by_fd(&self, fd: usize) -> Option<Arc<dyn File + Send + Sync>> {
        self.fd_table.lock().get_file_by_fd(fd).unwrap_or(None)
    }
    /// 获取当前进程的文件描述符表长度
    pub fn fd_table_len(&self) -> usize {
        self.fd_table.lock().table_len()
    }
    /// 判断是否打开文件描述符fd
    pub fn fd_is_none(&self, fd: usize) -> bool {
        self.fd_table.lock().table[fd].is_none()
    }
    /// 将fd作为index获取文件描述符
    pub fn get_fd(&self, fd: usize) -> Fd {
        self.fd_table.lock().get_fd(fd).unwrap()
    }
    /// 获取fd_table
    pub fn get_fd_table(&self) -> FdTable {
        self.fd_table.lock().clone()
    }
    /// 清空fd_table
    pub fn clear_fd_table(&self) {
        self.fd_table.lock().clear();
    }

    /// cwd
    /// 获取当前进程的当前工作目录
    pub fn get_current_path(&self) -> String {
        self.current_path.lock().clone()
    }
    /// 设置当前进程的当前工作目录
    pub fn set_current_path(&self, path: String) {
        *self.current_path.lock() = path;
    }

    /// waker
    /// 获取当前进程的waker
    pub fn task_waker(&self) -> Option<Waker> {
        unsafe { (*self.waker.get()).clone() }
    }
    /// 判断当前进程是否有waker
    pub fn has_waker(&self) -> bool {
        unsafe { (*self.waker.get()).is_some() }
    }
    /// 设置当前进程的waker
    pub fn set_task_waker(&self, waker: Waker) {
        unsafe { *self.waker.get() = Some(waker) }
    }
}