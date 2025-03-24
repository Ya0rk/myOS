use core::cell::SyncUnsafeCell;
use core::sync::atomic::{AtomicI32, AtomicUsize};
use core::task::Waker;

use super::{add_process_group_member, Fd, FdTable, TaskContext, ThreadGroup};
use super::{pid_alloc, KernelStack, Pid};
use crate::arch::shutdown;
use crate::fs::FileTrait;
use crate::mm::{translated_refmut, MapPermission, MemorySet};
use crate::sync::{new_shared, Shared, SpinNoIrqLock, TimeData};
use crate::syscall::CloneFlags;
use crate::task::{add_task, current_user_token, new_process_group, remove_task_by_pid, spawn_user_task, INITPROC};
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
    pid:            Pid,
    kernel_stack:   KernelStack,

    // 可变
    tgid:           AtomicUsize, // 线程组group_leader的 pid
    pgid:           AtomicUsize,
    task_status:    SpinNoIrqLock<TaskStatus>,

    base_size:      Shared<usize>,
    thread_group:   Shared<ThreadGroup>,
    memory_set:     Shared<MemorySet>,
    parent:         Shared<Option<Weak<TaskControlBlock>>>,
    pub children:   Shared<Vec<Arc<TaskControlBlock>>>,
    fd_table:       Shared<FdTable>,
    current_path:   Shared<String>,

    waker:          SyncUnsafeCell<Option<Waker>>,
    trap_cx:        SyncUnsafeCell<TrapContext>,
    /// 迟早会删
    task_cx:        SyncUnsafeCell<TaskContext>,
    time_data:      SyncUnsafeCell<TimeData>,
    child_cleartid: SyncUnsafeCell<Option<usize>>,

    exit_code:      AtomicI32,
}

impl TaskControlBlock {
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
        let tgid = pid_handle.0;
        let kernel_stack = KernelStack::new(&pid_handle);
        
        // 每个task有自己的kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack.get_kernel_stack_pos();
        memory_set.insert_framed_area(
            kernel_stack_bottom.into(), 
            kernel_stack_top.into(), 
            MapPermission::R | MapPermission::W,
        );
        
        // push a task context which goes to trap_return to the top of kernel stack
        let new_task = Arc::new(Self {
            pid: pid_handle,
            kernel_stack,
            
            // Shared
            pgid: AtomicUsize::new(0),
            tgid: AtomicUsize::new(tgid),
            task_status: SpinNoIrqLock::new(TaskStatus::Ready),
            base_size: new_shared(user_sp),
            thread_group: new_shared(ThreadGroup::new()),
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
            child_cleartid: SyncUnsafeCell::new(None),

            exit_code: AtomicI32::new(0),
        });

        debug!("initproc successfully created, pid: {}", new_task.get_pid());
        debug!("initproc entry: {:#x}, sp: {:#x}", entry_point, user_sp);

        new_task.add_thread_group_member(new_task.clone());
        new_process_group(new_task.get_pgid());
        add_task(&new_task);
        spawn_user_task(new_task.clone());

        new_task
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

    /// TODO:差分为thread 和 process new
    pub fn process_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(pid.0);
        let thread_group = new_shared(ThreadGroup::new());
        let task_status = SpinNoIrqLock::new(TaskStatus::Ready);
        let children = new_shared(Vec::new());
        let time_data = SyncUnsafeCell::new(TimeData::new());
        let exit_code = AtomicI32::new(0);
        let waker = SyncUnsafeCell::new(None);
        let parent = new_shared(Some(Arc::downgrade(self)));
        let current_path = self.current_path.clone();
        let child_cleartid = SyncUnsafeCell::new(None);
        let fd_table = match flag.contains(CloneFlags::CLONE_FILES) {
            true  => self.fd_table.clone(),
            false => new_shared(self.fd_table.lock().clone())
        };

        let kernel_stack = KernelStack::new(&pid);
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack.get_kernel_stack_pos();

        // modify kernel_sp in trap_cx
        let trap_cx = self.get_trap_cx_mut();
        trap_cx.set_kernel_sp(kernel_stack_top);
        let trap_cx = SyncUnsafeCell::new(*trap_cx);

        // TODO(YJJ):需要修改为clone和cow
        let mut child_memory_set = MemorySet::clone_from_existed_memset(&self.memory_set.lock());
        // let mut child_memory_set = self.memory_set.clone();
        child_memory_set.insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let memory_set = new_shared(child_memory_set);
        let task_cx = SyncUnsafeCell::new(TaskContext::goto_trap_loop(kernel_stack_top));

        let new_task = Arc::new(TaskControlBlock {
            pid,
            kernel_stack,

            // Shared
            pgid,
            tgid,
            base_size: self.base_size.clone(),
            thread_group,
            task_status,
            memory_set,
            parent,
            children,
            fd_table,
            current_path,

            // SyncUnsafeCell
            waker,
            trap_cx,
            task_cx,
            time_data,
            child_cleartid,
            exit_code,
        });
        // add child
        self.children.lock().push(new_task.clone());
        add_process_group_member(new_task.get_pgid(), new_task.get_pid());
        new_task.add_thread_group_member(new_task.clone());
        
        new_task
    }

    pub fn thread_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(self.get_tgid());
        let task_status = SpinNoIrqLock::new(TaskStatus::Ready);
        let thread_group = self.thread_group.clone();
        let memory_set = self.memory_set.clone();
        let parent = self.parent.clone();
        let children = self.children.clone();
        let current_path = self.current_path.clone();
        let waker = SyncUnsafeCell::new(None);
        let trap_cx = SyncUnsafeCell::new(*self.get_trap_cx());
        let time_data = SyncUnsafeCell::new(TimeData::new());
        let child_cleartid = SyncUnsafeCell::new(None);
        let exit_code = AtomicI32::new(0);
        let fd_table = match flag.contains(CloneFlags::CLONE_FILES) {
            true  => self.fd_table.clone(),
            false => new_shared(self.fd_table.lock().clone())
        };
        
        info!(
            "[fork]: child thread tid {}, parent process pid {}",
            pid, self.pid
        );

        let new_task = Arc::new(TaskControlBlock{
            kernel_stack: KernelStack::new(&pid),//
            pid,

            pgid,
            tgid,
            task_status,
            base_size: self.base_size.clone(),
            thread_group,
            memory_set,
            parent,
            children,
            fd_table,
            current_path,
            waker,
            trap_cx,
            time_data,
            child_cleartid,
            exit_code,
        
            task_cx: SyncUnsafeCell::new(TaskContext::zero_init()),//
        });

        new_task.add_thread_group_member(new_task.clone());

        new_task
    }
    
    pub fn exit(&self) {
        info!("Task {} exit;", self.get_pid());
        let pid = self.get_pid();

        // 如果是idle进程
        if pid == 0 {
            info!("Idle process exit with exit_code {} ...", self.get_exit_code());
            shutdown(false);
        }

        if let Some(tidaddress) = self.get_child_cleartid() {
            info!("[handle exit] clear child tid {:#x}", tidaddress);
            *translated_refmut( current_user_token(), tidaddress as *mut u32) = 0;
            // task.pcb_map(|proc| proc.futex_queue.wake(tidaddress as u32, 1));
            // task.futex_queue.lock().wake(tidaddress as u32, 1);
        }

        // TODO(YJJ):参考Phoneix
        if !self.is_leader() {   
            self.remove_thread_group_member(pid);
            remove_task_by_pid(pid);
        } else {
            // 将当前进程的子进程移动到initproc下
            for child in self.children.lock().iter() {
                child.set_parent(Some(Arc::downgrade(&INITPROC)));
                INITPROC.add_children(child.clone());
            }
            // TODO(YJJ):将信号发送给父进程，表示自己已经执行完成
        }
        
        self.clear_children();
        self.clear_fd_table();
        self.recycle_data_pages();
        self.set_zombie();
    }

    pub fn do_wait4(&self, pid: usize, wstatus: *mut i32) {
        let zombie_child = self.remove_child(pid);
        let exit_code = zombie_child.get_exit_code();
        // 将退出状态写入用户提供的指针
        if !wstatus.is_null() {
            *translated_refmut(self.get_user_token(), wstatus) = (exit_code & 0xff) << 8;
        }
        self.get_time_data_mut().update_child_time_when_exit();
        remove_task_by_pid(pid);
    }
}

impl TaskControlBlock {
    pub fn get_ppid(&self) -> usize {
        self.parent.lock().as_ref().map(|p| p.upgrade().unwrap().pid.0).unwrap_or(0)
    }
    /// 获取当前进程的pid
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    /// 获取time_data
    pub fn get_time_data(&self) -> &TimeData {
        unsafe { &*self.time_data.get() }
    }
    pub fn get_time_data_mut(&self) -> &mut TimeData {
        unsafe { &mut *(self.time_data.get() as *mut TimeData) }
    }

    /// 向线程组增加成员
    pub fn add_thread_group_member(&self, task: Arc<TaskControlBlock>) {
        self.thread_group.lock().add(task);
    }

    /// 删除线程组中的一个成员
    pub fn remove_thread_group_member(&self, pid: usize) {
        self.thread_group.lock().remove(pid.into());
    }

    /// 获取当前进程的pgid：组id
    pub fn get_pgid(&self) -> usize {
        self.pgid.load(core::sync::atomic::Ordering::Relaxed)
    }
    /// 设置当前进程的pgid
    pub fn set_pgid(&mut self, pgid: usize) {
        self.pgid.store(pgid, core::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_tgid(&self) ->usize {
        self.tgid.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_leader(&self) -> bool {
        self.get_pid() == self.get_tgid()
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
    pub fn remove_child(&self, pid: usize) -> Arc<TaskControlBlock>{
        self.children.lock().remove(pid)
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
    pub fn get_file_by_fd(&self, fd: usize) -> Option<Arc<dyn FileTrait + Send + Sync>> {
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

    pub fn get_child_cleartid(&self) -> Option<usize> {
        unsafe { *self.child_cleartid.get() }
    }
    /// 设置child_cleartid，参考：Linux系统编程手册500页
    pub fn set_child_cleartid(&self, ctid: usize) {
        unsafe { *self.child_cleartid.get() = Some(ctid) };
    }
}