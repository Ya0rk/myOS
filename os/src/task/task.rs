use core::cell::SyncUnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize};
use core::task::Waker;

use super::{add_proc_group_member, Fd, FdTable, TaskContext, ThreadGroup};
use super::{pid_alloc, KernelStack, Pid};
use crate::arch::shutdown;
use crate::fs::FileTrait;
use crate::mm::{translated_refmut, MapPermission, MemorySet};
use crate::signal::{SigActionFlag, SigCode, SigDetails, SigErr, SigHandler, SigInfo, SigMask, SigNom, SigPending, SigStruct, SignalStack};
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
    Stopped,
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

    base_size:      Shared<usize>, // 迟早要删
    thread_group:   Shared<ThreadGroup>,
    memory_set:     Shared<MemorySet>,
    parent:         Shared<Option<Weak<TaskControlBlock>>>,
    pub children:   Shared<Vec<Arc<TaskControlBlock>>>,
    fd_table:       Shared<FdTable>,
    current_path:   Shared<String>,

    // signal
    pub pending:        AtomicBool, // 表示是否有sig在等待，用于快速检查是否需要处理信号
    pub ucontext:       AtomicUsize, // ucontext指针，保存用户数据，在sigreturn需要用到来恢复用户环境
    pub sig_pending:    SpinNoIrqLock<SigPending>, // signal 等待队列
    pub blocked:        SyncUnsafeCell<SigMask>,   // 信号屏蔽字,表明进程不处理的信号
    pub handler:        Shared<SigStruct>, // 表示信号相应的处理方法,一共64个信号
    pub sig_stack:      SyncUnsafeCell<Option<SignalStack>>, // 信号栈，保存信号栈信息


    waker:          SyncUnsafeCell<Option<Waker>>,
    trap_cx:        SyncUnsafeCell<TrapContext>,
    task_cx:        SyncUnsafeCell<TaskContext>,// 迟早会删
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

            pending: AtomicBool::new(false),
            ucontext: AtomicUsize::new(0),
            sig_pending: SpinNoIrqLock::new(SigPending::new()),
            blocked: SyncUnsafeCell::new(SigMask::empty()),
            handler: new_shared(SigStruct::new()),
            sig_stack: SyncUnsafeCell::new(None),
            
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

        // 重置自定义的信号处理
        self.handler.lock().flash_signal_handlers();


        debug!("task.exec.pid={}", self.pid.0);
    }

    pub fn process_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(pid.0);
        let pending = AtomicBool::new(false);
        let ucontext = AtomicUsize::new(0);
        let sig_pending = SpinNoIrqLock::new(SigPending::new());
        let blocked = SyncUnsafeCell::new(self.get_blocked().clone());
        let sig_stack = SyncUnsafeCell::new(None);
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
        let sig = match flag.contains(CloneFlags::SIGCHLD) {
            true  => self.handler.clone(), // 和父进程共享，只是增加父进程sig的引用计数，效率高
            false => new_shared(self.handler.lock().clone()), // 一个新的副本，子进程可以独立修改
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

            pending,
            ucontext,
            sig_pending,
            blocked,
            handler: sig,
            sig_stack,

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
        add_proc_group_member(new_task.get_pgid(), new_task.get_pid());
        new_task.add_thread_group_member(new_task.clone());
        
        new_task
    }

    pub fn thread_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(self.get_tgid());
        let pending= AtomicBool::new(false);
        let ucontext = AtomicUsize::new(0);
        let sig_pending = SpinNoIrqLock::new(SigPending::new());
        let blocked = SyncUnsafeCell::new(self.get_blocked().clone());
        let sig_stack = SyncUnsafeCell::new(None);
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
        let sig = match flag.contains(CloneFlags::SIGCHLD) {
            true  => self.handler.clone(), // 和父进程共享，只是增加父进程sig的引用计数，效率高
            false => new_shared(self.handler.lock().clone()), // 一个新的副本，子进程可以独立修改
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

            pending,
            ucontext,
            sig_pending,
            blocked,
            handler: sig,
            sig_stack,

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
    
    pub fn do_exit(&self) {
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

        if !self.is_leader() {   
            self.remove_thread_group_member(pid);
            remove_task_by_pid(pid);
        } else {
            // 将当前进程的子进程移动到initproc下
            if !self.children.lock().is_empty(){
                for child in self.children.lock().iter() {
                    if child.is_zombie() {
                        let sig_info = SigInfo::new(
                            SigNom::SIGCHLD, 
                            SigCode::CLD_EXITED, 
                            SigErr::empty(), 
                            SigDetails::Chld { 
                                pid: child.get_pid(), 
                                status: child.get_status(), 
                                exit_code: child.get_exit_code()
                            }
                        );
                        INITPROC.proc_recv_siginfo(sig_info);
                    }
                    child.set_parent(Some(Arc::downgrade(&INITPROC)));
                    INITPROC.add_children(child.clone());
                }
                self.clear_children();
            }
            // 当前是leader，需要将信号发送给leader的父进程，表示自己已经执行完成
            match self.get_parent() {
                Some(parent) => {
                    let sig_info = SigInfo::new(
               SigNom::SIGCHLD, 
                SigCode::CLD_EXITED, 
                 SigErr::empty(), 
                 SigDetails::Chld { 
                            pid, 
                            status: self.get_status(), 
                            exit_code: self.get_exit_code()
                        }
                    );
                    parent.proc_recv_siginfo(sig_info);
                }
                None => panic!("this proc has no parent!"),
            }
        }
        
        self.clear_fd_table();
        self.recycle_data_pages();        
    }

    pub fn do_wait4(&self, pid: usize, wstatus: *mut i32, exit_code: i32) {
        let zombie_child = self.remove_child(pid);
        // 将退出状态写入用户提供的指针
        if !wstatus.is_null() {
            *translated_refmut(self.get_user_token(), wstatus) = (exit_code & 0xff) << 8;
        }
        let (utime, stime) = zombie_child.get_time_data().get_ustime();
        self.get_time_data_mut().update_child_time_when_exit(utime, stime);
        remove_task_by_pid(pid);
    }

    /// 为task设置可以被唤醒的信号，当task接收到这些信号时，会被唤醒
    pub fn set_wake_up_signal(&self, signal: SigMask) {
        let mut sig_pending = self.sig_pending.lock();
        sig_pending.need_wake = signal;
    }

    /// 通知父进程
    pub fn do_notify_parent(self: &Arc<Self>, si_signo: SigNom, si_code: SigCode) {
        let parent = self.get_parent().expect("this has no parent!");
        let p_handler = parent
            .handler
            .lock()
            .fetch_signal_handler(SigNom::SIGCHLD as usize);

        if si_signo == SigNom::SIGCHLD
            && ( p_handler.sa.sa_handler == SigHandler::SIG_IGN
            ||  p_handler.sa.sa_flags.contains(SigActionFlag::SA_NOCLDSTOP) ) 
        {
            return ;
        }

        let sig_info = SigInfo::new(
            si_signo, 
            si_code, 
            SigErr::from_bits(0).unwrap(), 
            SigDetails::None
        );

        parent.thread_recv_siginfo(sig_info);
    }

    /// 进程级信号:
    /// 
    /// 发送给整个线程组的（例如 kill -INT <pid>），必须保证至少有一个线程能处理它
    /// 
    /// 随机选择一个没有阻塞当前信号的线程来接受信号
    /// 
    /// 避免信号风暴或信号丢失
    pub fn proc_recv_siginfo(&self, sig_info: SigInfo) {
        debug_assert!(self.is_leader());
        let tg = self.thread_group.lock();

        // 特权信号（SIGKILL/SIGSTOP）：
        // 应绕过阻塞检查，直接递送给任意线程（需特殊判断）：
        if sig_info.signo == SigNom::SIGKILL 
            || sig_info.signo == SigNom::SIGSTOP 
        {
            let thread = tg.tasks.values().next().unwrap().upgrade().unwrap();
            thread.thread_recv_siginfo(sig_info);
            return;
        }

        // 尝试寻找未阻塞信号的线程
        for (_, task) in tg.tasks.iter() {
            if let Some(thread) = task.upgrade() {
                if !thread.get_blocked().have(sig_info.signo as usize) {
                    thread.thread_recv_siginfo(sig_info);
                    return;
                }
            }
        }

        let thread = tg.tasks.iter().next().unwrap().1.upgrade().unwrap();
        thread.thread_recv_siginfo(sig_info);
    }

    /// 线程级信号:
    /// 
    /// 通过 pthread_kill(tid, sig) 或 tgkill(pid, tid, sig) 发送给特定线程
    /// 
    /// 直接递送给目标线程，无视其他线程
    pub fn thread_recv_siginfo(&self, sig_info: SigInfo) {
        let mut sig_pending = self.sig_pending.lock();
        sig_pending.add(sig_info);
        self.set_pending(true);
        if sig_pending.need_wake.have(sig_info.signo as usize) {
            self.wake_up();
        }
    }
}

impl TaskControlBlock {
    /// 检测pending字段，判断是否有信号需要处理
    pub fn pending(&self) -> bool {
        self.pending.load(core::sync::atomic::Ordering::Relaxed)
    }
    /// 设置pending，代表 是否 有信号等待被处理
    pub fn set_pending(&self, value: bool) {
        self.pending.store(value, core::sync::atomic::Ordering::Relaxed);
    }

    /// 取出task中的sig stack 留下None
    pub fn get_sig_stack_mut(&self) -> &mut Option<SignalStack> {
        unsafe { &mut *self.sig_stack.get() }
    }

    /// 获取信号屏蔽字段
    pub fn get_blocked(&self) -> &SigMask {
        unsafe { &*self.blocked.get() }
    }
    /// 获取信号屏蔽字段的mut
    pub fn get_blocked_mut(&self) -> &mut SigMask {
        unsafe { &mut *self.blocked.get() }
    }

    /// 设置ucontext
    pub fn set_ucontext(&self, addr: usize) {
        self.ucontext.store(addr, core::sync::atomic::Ordering::Relaxed);
    }
    /// 获取ucontext
    pub fn get_ucontext(&self) -> usize {
        self.ucontext.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// 获取parent的pid
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
        unsafe { &mut *self.time_data.get() }
    }

    /// 向线程组增加成员
    pub fn add_thread_group_member(&self, task: Arc<TaskControlBlock>) {
        self.thread_group.lock().add(task);
    }

    /// 删除线程组中的一个成员
    pub fn remove_thread_group_member(&self, pid: usize) {
        self.thread_group.lock().remove(pid.into());
    }

    /// 将所有子线程设置为zombie
    pub fn kill_all_thread(self: &Arc<Self>) {
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            thread.upgrade().unwrap().set_zombie();
        }
    }

    /// 唤醒当前的进程
    pub fn wake_up(&self) {
        self.get_waker()
            .as_ref()
            .expect("this task has no waker!")
            .wake_by_ref();
    }

    /// 将所有子线程设置为stopped挂起
    pub fn stop_all_thread(self: &Arc<Self>) {
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            let thread = thread.upgrade().unwrap();
            thread.set_stopped();
            thread.set_wake_up_signal(SigMask::SIGCONT | SigMask::SIGKILL | SigMask::SIGSTOP);
        }
        self.do_notify_parent(SigNom::SIGCHLD, SigCode::CLD_STOPPED);
    }

    /// 当收到SIGCONT时，将stopped的子线程设置为running
    pub fn cont_all_thread(self: &Arc<Self>) {
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            let thread = thread.upgrade().unwrap();
            thread.set_running();
            self.wake_up();
        }
        self.do_notify_parent(SigNom::SIGCHLD, SigCode::CLD_CONTINUED);
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
    pub fn set_stopped(&self) {
        *self.task_status.lock() = TaskStatus::Stopped;
    }
    pub fn is_stopped(&self) -> bool {
        self.get_status() == TaskStatus::Stopped
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
    /// 获取父进程的Arc引用，不是弱引用
    pub fn get_parent(&self) -> Option<Arc<TaskControlBlock>> {
        self.parent.lock().clone().unwrap().upgrade()
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
    pub fn get_waker(&self) -> &Option<Waker> {
        unsafe { & *self.waker.get() }
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