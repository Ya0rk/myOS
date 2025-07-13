use super::{
    add_proc_group_member, remove_proc_group_member, FdInfo, FdTable, FutexBucket, RobustList,
    ShmidTable, ThreadGroup,
};
use super::{pid_alloc, Pid};
use crate::fs::ext4::NormalFile;
use crate::fs::{init, FileClass, FileTrait};
use crate::hal::arch::{sfence, shutdown};
use crate::hal::config::INITPROC_PID;
use crate::hal::trap::TrapContext;
use crate::ipc::shm::SHARED_MEMORY_MANAGER;
use crate::ipc::IPCKey;
use crate::mm::address::VirtAddr;
use crate::mm::memory_space::vm_area::{VmArea, VmAreaType};
use crate::mm::memory_space::{create_elf_tables, vm_area::MapPerm, MemorySpace};
use crate::mm::{memory_space};
use crate::signal::{
    SigActionFlag, SigCode, SigDetails, SigErr, SigHandlerType, SigInfo, SigMask, SigNom,
    SigPending, SigStruct, SignalStack,
};
use crate::sync::time::ITimerVal;
use crate::sync::{get_waker, new_shared, Shared, SpinNoIrqLock, TimeData};
use crate::syscall::{CloneFlags, CpuSet, RLimit64, SchedParam};
use crate::task::manager::get_init_proc;
use crate::task::{
    add_task, current_task, current_user_token, get_task_by_pid, new_process_group,
    remove_task_by_pid, spawn_user_task, FutexHashKey, FutexOp, FUTEX_BITSET_MATCH_ANY,
};
use crate::utils::SysResult;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::SyncUnsafeCell;
use core::fmt::{Debug, Display};
use core::future::Ready;
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize};
use core::task::Waker;
use core::time::Duration;
use log::{debug, info};
use xmas_elf::dynamic;

pub struct TaskControlBlock {
    // 不可变
    pub pid: Pid,

    // 可变
    pub tgid: AtomicUsize, // 所属线程组的leader的 pid，如果自己是leader，那tgid = pid
    pub pgid: AtomicUsize, // 所属进程组id号
    pub task_status: SpinNoIrqLock<TaskStatus>,

    pub thread_group: Shared<ThreadGroup>,
    pub memory_space: SyncUnsafeCell<Shared<MemorySpace>>,
    pub parent: Shared<Option<Weak<TaskControlBlock>>>,
    pub children: Shared<BTreeMap<usize, Arc<TaskControlBlock>>>,
    pub fd_table: Shared<FdTable>,
    pub current_path: Shared<String>,
    pub robust_list: Shared<RobustList>,
    pub futex_list: Shared<FutexBucket>,
    pub itimers: Shared<[ITimerVal; 3]>, // 三个定时器，分别对应SIGALRM, SIGVTALRM, SIGPROF
    pub fsz_limit: Shared<Option<RLimit64>>, // 记录当前进程最大文件大小

    /// to record shm ids (same as ipc key) to manage detaching at exit and execve
    pub shmid_table: Shared<ShmidTable>,

    // signal
    pub pending: AtomicBool, // 表示是否有sig在等待，用于快速检查是否需要处理信号
    pub ucontext: AtomicUsize, // ucontext指针，保存用户数据，在sigreturn需要用到来恢复用户环境
    pub sig_pending: SpinNoIrqLock<SigPending>, // signal 等待队列
    pub blocked: SyncUnsafeCell<SigMask>, // 信号屏蔽字,表明进程不处理的信号
    pub handler: Shared<SigStruct>, // 表示信号相应的处理方法,一共64个信号
    pub sig_stack: SyncUnsafeCell<Option<SignalStack>>, // 信号栈，保存信号栈信息

    pub waker: SyncUnsafeCell<Option<Waker>>,
    pub trap_cx: SyncUnsafeCell<TrapContext>,
    pub time_data: SyncUnsafeCell<TimeData>,
    pub clear_child_tid: SyncUnsafeCell<Option<usize>>,
    pub set_child_tid: SyncUnsafeCell<Option<usize>>,
    pub cpuset: SyncUnsafeCell<CpuSet>,
    pub prio: SyncUnsafeCell<SchedParam>,

    pub exit_code: AtomicI32,
}

impl TaskControlBlock {
    /// 创建新task,只有initproc会调用
    pub async fn new(elf_file: Arc<dyn FileTrait>) -> Arc<Self> {
        let (mut memory_space, entry_point, sp_init, auxv) =
            MemorySpace::new_user_from_elf(elf_file).await.expect("Failed on mapping initproc");
        info!("entry point: {:#x}", entry_point);

        unsafe { memory_space.switch_page_table() };
        crate::hal::arch::set_sum();
        // unsafe{riscv::register::sstatus::set_sum();}
        let (user_sp, argc, argv_p, env_p) =
            create_elf_tables(sp_init.into(), Vec::new(), Vec::new(), auxv);
        let trap_cx = TrapContext::app_init_context(entry_point, user_sp);
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let tgid = pid_handle.0;

        // push a task context which goes to trap_return to the top of kernel stack
        let new_task = Arc::new(Self {
            pid: pid_handle,

            // Shared
            pgid: AtomicUsize::new(0),
            tgid: AtomicUsize::new(tgid),
            task_status: SpinNoIrqLock::new(TaskStatus::Ready),
            thread_group: new_shared(ThreadGroup::new()),
            memory_space: SyncUnsafeCell::new(new_shared(memory_space)),
            parent: new_shared(None),
            children: new_shared(BTreeMap::new()),
            fd_table: new_shared(FdTable::new()),
            current_path: new_shared(String::from("/")), // root directory
            robust_list: new_shared(RobustList::new()),
            futex_list: new_shared(FutexBucket::new()),
            itimers: new_shared([ITimerVal::default(); 3]),
            fsz_limit: new_shared(None),

            shmid_table: new_shared(ShmidTable::new()),

            pending: AtomicBool::new(false),
            ucontext: AtomicUsize::new(0),
            sig_pending: SpinNoIrqLock::new(SigPending::new()),
            blocked: SyncUnsafeCell::new(SigMask::empty()),
            handler: new_shared(SigStruct::new()),
            sig_stack: SyncUnsafeCell::new(None),

            // SyncUnsafeCell
            waker: SyncUnsafeCell::new(None),
            trap_cx: SyncUnsafeCell::new(trap_cx),
            time_data: SyncUnsafeCell::new(TimeData::new()),
            clear_child_tid: SyncUnsafeCell::new(None),
            set_child_tid: SyncUnsafeCell::new(None),
            cpuset: SyncUnsafeCell::new(CpuSet::default()),
            prio: SyncUnsafeCell::new(SchedParam::default()),

            exit_code: AtomicI32::new(0),
        });

        debug!("initproc successfully created, pid: {}", new_task.get_pid());
        debug!("initproc entry: {:#x}, sp: {:#x}", entry_point, user_sp);

        new_task.add_thread_group_member(new_task.clone());
        new_process_group(new_task.get_pgid(), new_task.get_pid());
        add_task(&new_task);
        spawn_user_task(new_task.clone());
        info!("spawn init proc");

        new_task
    }
    pub async fn execve(&self, elf_file: Arc<dyn FileTrait>, argv: Vec<String>, env: Vec<String>) {
        info!("execve start");
        // info!("[execve] argv:{:?}, env:{:?}", argv, env);
        let (mut memory_space, entry_point, sp_init, auxv) =
            MemorySpace::new_user_from_elf_lazily(elf_file).await
            .expect("[execve] Failed on mapping elf file");

        // info!("execve memory_set created");

        // 终止所有子线程
        for (_, weak_task) in self.thread_group.lock().tasks.iter() {
            let task = weak_task.upgrade().unwrap();
            if !task.is_leader() {
                info!("[exec]: terminate task, pid = {}", task.get_pid());
                task.set_zombie();
            }
        }

        // **** access inner exclusively
        // substitute memory_set
        unsafe { memory_space.switch_page_table() };
        // let mut mem = self.memory_space.lock();
        // *mem = memory_space;
        *self.get_memory_space_mut() = new_shared(memory_space);
        self.detach_all_shm();
        let (user_sp, argc, argv_p, env_p) = create_elf_tables(sp_init.into(), argv, env, auxv);

        // set trap cx
        let trap_cx = self.get_trap_cx_mut();
        trap_cx.set_sepc(entry_point);
        trap_cx.set_sp(user_sp);
        trap_cx.set_arg(argc, argv_p, env_p);
        // 判断是否有O_CLOEXEC，如果有的话就清空当前位置的fd，避免子进程使用一些父进程的fd
        self.fd_table.lock().close_on_exec();
        // 重置自定义的信号处理
        self.handler.lock().flash_signal_handlers();
        unsafe { *self.sig_stack.get() = None };
        debug_point!("");

        debug!("task.exec.pid={}", self.pid.0);
    }

    pub fn do_process_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        info!("[process_fork] start, flags = {:?}", flag);
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(pid.0);
        let pending = AtomicBool::new(false);
        let ucontext = AtomicUsize::new(0);
        let sig_pending = SpinNoIrqLock::new(SigPending::new());
        let blocked = SyncUnsafeCell::new(self.get_blocked().clone());
        let sig_stack = SyncUnsafeCell::new(None);
        let shmid_table = new_shared(self.shmid_table.lock().clone());
        let thread_group = new_shared(ThreadGroup::new());
        let task_status = SpinNoIrqLock::new(TaskStatus::Ready);
        let children = new_shared(BTreeMap::new());
        let fsz_limit = new_shared(None);
        let time_data = SyncUnsafeCell::new(TimeData::new());
        let exit_code = AtomicI32::new(0);
        let waker = SyncUnsafeCell::new(None);
        let parent = new_shared(Some(Arc::downgrade(self)));
        let current_path = new_shared(self.current_path.lock().clone());
        let robust_list = new_shared(RobustList::new());
        let clear_child_tid = SyncUnsafeCell::new(None);
        let set_child_tid = SyncUnsafeCell::new(None);
        let cpuset = SyncUnsafeCell::new(CpuSet::default());
        let prio = SyncUnsafeCell::new(SchedParam::default());
        let futex_list = new_shared(FutexBucket::new());
        let itimers = new_shared([ITimerVal::default(); 3]);
        let fd_table = match flag.contains(CloneFlags::CLONE_FILES) {
            true => self.fd_table.clone(),
            false => new_shared(self.fd_table.lock().clone()),
        };
        let sig = match flag.contains(CloneFlags::CLONE_SIGHAND) {
            // 修改这里的致命判断错误
            true => {
                info!("[process_fork] sigchld");
                self.handler.clone() // 和父进程共享，只是增加父进程sig的引用计数，效率高
            }
            false => {
                info!("[process_fork] sigchld false");
                // 一个新的副本，子进程可以独立修改
                new_shared(self.handler.lock().clone())
            }
        };

        // modify kernel_sp in trap_cx
        let trap_cx = self.get_trap_cx_mut();
        let trap_cx = SyncUnsafeCell::new(*trap_cx);

        let memory_space = match flag.contains(CloneFlags::CLONE_VM) {
            true => SyncUnsafeCell::new(self.get_memory_space().clone()),
            false => {
                info!("[process_fork] self memoty space.");
                let child_memory_space =
                    MemorySpace::from_user_lazily(&mut self.get_memory_space().lock());
                unsafe {
                    sfence();
                }
                SyncUnsafeCell::new(new_shared(child_memory_space))
            }
        };

        let new_task = Arc::new(TaskControlBlock {
            pid,

            // Shared
            pgid,
            tgid,
            thread_group,
            task_status,
            memory_space,
            parent,
            children,
            fd_table,
            current_path,
            robust_list,
            futex_list,
            itimers,
            fsz_limit,

            shmid_table,

            pending,
            ucontext,
            sig_pending,
            blocked,
            handler: sig,
            sig_stack,

            // SyncUnsafeCell
            waker,
            trap_cx,
            time_data,
            clear_child_tid,
            set_child_tid,
            cpuset,
            prio,
            exit_code,
        });
        // add child
        self.add_child(new_task.clone());
        add_proc_group_member(new_task.get_pgid(), new_task.get_pid());
        new_task.add_thread_group_member(new_task.clone());
        info!(
            "process fork success, new pid = {}, parent pid = {}",
            new_task.get_pid(),
            new_task.get_parent().unwrap().get_pid()
        );

        new_task
    }

    pub fn do_thread_fork(self: &Arc<Self>, flag: CloneFlags) -> Arc<Self> {
        info!("[thread_fork] start, flags = {:?}", flag);
        let pid = pid_alloc();
        let pgid = AtomicUsize::new(self.get_pgid());
        let tgid = AtomicUsize::new(self.get_tgid());
        let pending = AtomicBool::new(false);
        let ucontext = AtomicUsize::new(0);
        let fsz_limit = self.fsz_limit.clone();
        let sig_pending = SpinNoIrqLock::new(SigPending::new());
        let blocked = SyncUnsafeCell::new(self.get_blocked().clone());
        let sig_stack = SyncUnsafeCell::new(None);
        let shmid_table = new_shared(self.shmid_table.lock().clone());
        let task_status = SpinNoIrqLock::new(TaskStatus::Ready);
        let thread_group = self.thread_group.clone();
        let parent = self.parent.clone();
        let children = self.children.clone();
        let current_path = self.current_path.clone();
        let robust_list = self.robust_list.clone();
        let waker = SyncUnsafeCell::new(None);
        let trap_cx = SyncUnsafeCell::new(*self.get_trap_cx());
        let time_data = SyncUnsafeCell::new(TimeData::new());
        let clear_child_tid = SyncUnsafeCell::new(None);
        let set_child_tid = SyncUnsafeCell::new(None);
        let cpuset = SyncUnsafeCell::new(CpuSet::default());
        let prio = SyncUnsafeCell::new(SchedParam::default());
        let exit_code = AtomicI32::new(0);
        let futex_list = self.futex_list.clone();
        let itimers = self.itimers.clone();
        let fd_table = match flag.contains(CloneFlags::CLONE_FILES) {
            // 修改这里的致命错误
            true => self.fd_table.clone(),
            false => new_shared(self.fd_table.lock().deref().clone()),
        };
        let sig = match flag.contains(CloneFlags::CLONE_SIGHAND) {
            true => {
                info!("[thread_fork] sigchld");
                self.handler.clone() // 和父进程共享，只是增加父进程sig的引用计数，效率高
            }
            false => {
                info!("[thread_fork] sigchld false");
                // 一个新的副本，子进程可以独立修改
                new_shared(self.handler.lock().clone())
            }
        };

        let memory_space = match flag.contains(CloneFlags::CLONE_VM) {
            true => SyncUnsafeCell::new(self.get_memory_space().clone()),
            false => {
                let child_memory_space =
                    MemorySpace::from_user_lazily(&mut self.get_memory_space().lock());
                unsafe {
                    sfence();
                }
                SyncUnsafeCell::new(new_shared(child_memory_space))
            }
        };

        info!(
            "[fork]: child thread tid {}, parent process pid {}",
            pid, self.pid
        );

        let new_task = Arc::new(TaskControlBlock {
            pid,

            pgid,
            tgid,

            pending,
            ucontext,
            sig_pending,
            blocked,
            handler: sig,
            sig_stack,
            robust_list,
            futex_list,
            itimers,
            fsz_limit,

            shmid_table,

            task_status,
            thread_group,
            memory_space,
            parent,
            children,
            fd_table,
            current_path,
            waker,
            trap_cx,
            time_data,
            clear_child_tid,
            set_child_tid,
            cpuset,
            prio,
            exit_code,
        });

        new_task.add_thread_group_member(new_task.clone());

        new_task
    }

    pub fn do_exit(&self) {
        info!("[do_exit] Task pid = {} exit;", self.get_pid());
        // println!("[do_exit] Task pid = {} exit;", self.get_pid());
        let pid = self.get_pid();

        if let Some(tidaddress) = self.get_child_cleartid() {
            info!("[handle exit] clear child tid {:#x}", tidaddress);
            unsafe {
                // 这里需要检测地址空间是否还有其他使用者，如果存在就需要清零tidaddress
                // 在pthread中tidaddress会被父线程或者其他线程 通过futex系统调用 轮循监测(poll)
                // 如果tidaddress被清零了，其他线程会检测到这里的值改变，那么说明有线程退出
                // 在futex中会执行wake唤醒其他线程
                if Arc::strong_count(self.get_memory_space()) > 1 {
                    // println!("[handle exit] clear child tid {:#x}, task id {}", tidaddress, pid);
                    core::ptr::write(tidaddress as *mut usize, 0);
                    let key = FutexHashKey::get_futex_key(tidaddress, FutexOp::empty());
                    let mut binding = self.futex_list.lock();
                    binding.to_wake(key, FUTEX_BITSET_MATCH_ANY, 1);
                    let key = FutexHashKey::get_futex_key(tidaddress, FutexOp::FUTEX_PRIVATE);
                    binding.to_wake(key, FUTEX_BITSET_MATCH_ANY, 1);
                }
                // core::ptr::write(tidaddress as *mut usize, 0);
                *self.clear_child_tid.get() = None; // 清除清理子线程的地址
            }
        }

        if !self.is_leader() {
            self.children.lock().clear();
            self.remove_thread_group_member(pid);
            info!(
                "[do_exit] task is not leader, threadgroup len = {}",
                self.thread_group.lock().tasks.len()
            );
            remove_task_by_pid(pid);
            return; // 致命错误，这里是处理线程的分支
        }

        // 将当前进程的子进程移动到initproc下
        // info!("[do_exit] task is leader");
        let mut lock_child = self.children.lock();
        if !lock_child.is_empty() {
            info!("[do_exit] task has child");
            let init_proc = get_init_proc();
            for (child_pid, child) in lock_child.iter() {
                if child.is_zombie() {
                    info!("[do_exit] child pdi = {} is zmobie", child_pid);
                    child.exit_notify(&init_proc);
                }
                child.set_parent(Some(Arc::downgrade(&init_proc)));
            }
            init_proc.children.lock().extend(lock_child.clone());
            lock_child.clear();
        }
        drop(lock_child);
        // 当前是leader，需要将信号发送给leader的父进程，表示自己已经执行完成
        match self.get_parent() {
            Some(parent) => {
                info!(
                    "[do_exit] task to info parent pid = {}, exit code = {}",
                    parent.get_pid(),
                    self.get_exit_code()
                );
                self.exit_notify(&parent);
            }
            None => {
                use log::error;
                error!("proc {} has no parent!", self.get_pid());
            }
        }

        // self.remove_thread_group_member(pid);
        self.clear_fd_table();
        self.detach_all_shm();
        self.recycle_data_pages();
    }

    /// 向父进程发送信号通知
    fn exit_notify(&self, parent: &Arc<TaskControlBlock>) {
        let sig_info = SigInfo::new(
            SigNom::SIGCHLD,
            SigCode::CLD_EXITED,
            SigErr::empty(),
            SigDetails::Chld {
                pid: self.get_pid(),
                status: self.get_status(),
                exit_code: self.get_exit_code(),
            },
        );
        parent.proc_recv_siginfo(sig_info);
    }

    pub fn do_wait4(&self, pid: usize, wstatus: usize, exit_code: i32) {
        let zombie_child = self.remove_child(pid);
        // 将退出状态写入用户提供的指针
        if wstatus != 0 {
            let wstatus = wstatus as *mut i32;
            unsafe { wstatus.write_volatile(exit_code) };
            // *translated_refmut(self.get_user_token(), wstatus) = (exit_code & 0xff) << 8;
        }
        let (utime, stime) = zombie_child.get_time_data().get_ustime();
        self.get_time_data_mut()
            .update_child_time_when_exit(utime, stime);
        remove_task_by_pid(pid);
        remove_proc_group_member(zombie_child.get_pgid(), pid);
    }

    /// 为task设置可以被唤醒的信号，当task接收到这些信号时，会被唤醒
    pub fn set_wake_up_signal(&self, signal: SigMask) {
        let mut sig_pending = self.sig_pending.lock();
        sig_pending.need_wake = signal | SigMask::SIGKILL | SigMask::SIGSTOP;
    }

    /// 通知父进程
    pub fn do_notify_parent(self: &Arc<Self>, si_signo: SigNom, si_code: SigCode) {
        let parent = self.get_parent().expect("this has no parent!");
        let p_handler = parent
            .handler
            .lock()
            .fetch_signal_handler(SigNom::SIGCHLD as usize);

        if si_signo == SigNom::SIGCHLD
            && (p_handler.sa_type == SigHandlerType::IGNORE
                || p_handler.sa.sa_flags.contains(SigActionFlag::SA_NOCLDSTOP))
        {
            return;
        }

        let sig_info = SigInfo::new(
            si_signo,
            si_code,
            SigErr::from_bits(0).unwrap(),
            SigDetails::None,
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
        if sig_info.signo == SigNom::SIGKILL || sig_info.signo == SigNom::SIGSTOP {
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

    pub fn check_shutdown(&self) {
        if self.get_pid() == INITPROC_PID {
            info!(
                "init process exit with exit_code {} ...",
                self.get_exit_code()
            );
            shutdown(false);
        }
    }
}

impl TaskControlBlock {
    pub fn whit_itimers<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut [ITimerVal; 3]) -> R,
    {
        let mut itimers = self.itimers.lock();
        f(&mut itimers)
    }

    /// 获取cpuset
    pub fn get_cpuset(&self) -> &CpuSet {
        unsafe { &*self.cpuset.get() }
    }
    /// 设置cpuset
    pub fn set_cpuset(&self, cpuset: CpuSet) {
        unsafe {
            *self.cpuset.get() = cpuset;
        }
    }

    /// 检测pending字段，判断是否有信号需要处理
    pub fn pending(&self) -> bool {
        self.pending.load(core::sync::atomic::Ordering::SeqCst)
    }
    /// 设置pending，代表 是否 有信号等待被处理
    pub fn set_pending(&self, value: bool) {
        self.pending
            .store(value, core::sync::atomic::Ordering::SeqCst);
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
    /// 设置信号屏蔽字段
    pub fn set_blocked(&self, mask: SigMask) {
        unsafe { *self.blocked.get() = mask };
    }

    /// 设置ucontext
    pub fn set_ucontext(&self, addr: usize) {
        self.ucontext
            .store(addr, core::sync::atomic::Ordering::SeqCst);
    }
    /// 获取ucontext
    pub fn get_ucontext(&self) -> usize {
        self.ucontext.load(core::sync::atomic::Ordering::SeqCst)
    }

    /// 获取parent的pid
    pub fn get_ppid(&self) -> usize {
        self.parent
            .lock()
            .as_ref()
            .map(|p| p.upgrade().unwrap().pid.0)
            .unwrap_or(0)
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

    pub fn get_prio(&self) -> &SchedParam {
        unsafe { &*self.prio.get() }
    }
    pub fn get_prio_mut(&self) -> &mut SchedParam {
        unsafe { &mut *self.prio.get() }
    }

    /// 向线程组增加成员
    pub fn add_thread_group_member(&self, task: Arc<TaskControlBlock>) {
        self.thread_group.lock().add(task);
    }
    /// 删除线程组中的一个成员
    pub fn remove_thread_group_member(&self, pid: usize) {
        self.thread_group.lock().remove(pid);
    }

    /// 将所有子线程设置为zombie
    pub fn kill_all_thread(self: &Arc<Self>) {
        let lock_thread_gp = self.thread_group.lock();
        for (_, thread) in lock_thread_gp.tasks.iter() {
            let task = match thread.upgrade() {
                Some(a) => a,
                None => continue,
            };
            task.set_zombie();
        }
    }

    /// 唤醒当前的进程
    pub fn wake_up(&self) {
        self.get_task_waker()
            .as_ref()
            .expect("this task has no waker!")
            .wake_by_ref();
    }

    /// 将所有子线程设置为stopped挂起
    pub fn stop_all_thread(self: &Arc<Self>, signo: SigNom) {
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            let thread = thread.upgrade().unwrap();
            thread.set_stopped();
            thread.set_wake_up_signal(SigMask::SIGCONT | SigMask::SIGKILL | SigMask::SIGSTOP);
        }
        // self.do_notify_parent(signo, SigCode::CLD_STOPPED);
        self.do_notify_parent(SigNom::SIGCHLD, SigCode::CLD_STOPPED);
    }

    /// 当收到SIGCONT时，将stopped的子线程设置为running
    pub fn cont_all_thread(self: &Arc<Self>, signo: SigNom) {
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            let thread = thread.upgrade().unwrap();
            thread.set_running();
            self.wake_up();
        }
        // self.do_notify_parent(signo, SigCode::CLD_CONTINUED);
        self.do_notify_parent(SigNom::SIGCHLD, SigCode::CLD_CONTINUED);
    }

    /// 进程收到kill或者stop信号
    pub fn rv_intr(&self) -> bool {
        let (res, _, _) = self
            .sig_pending
            .lock()
            .has_expected(SigMask::SIGKILL | SigMask::SIGSTOP);
        res
    }

    /// 获取当前进程的pgid：组id
    pub fn get_pgid(&self) -> usize {
        self.pgid.load(core::sync::atomic::Ordering::SeqCst)
    }
    /// 设置当前进程的pgid
    pub fn set_pgid(&self, pgid: usize) {
        self.pgid.store(pgid, core::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_tgid(&self) -> usize {
        self.tgid.load(core::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_leader(&self) -> bool {
        self.get_pid() == self.get_tgid()
    }

    pub fn get_user_token(&self) -> usize {
        self.get_memory_space().lock().token()
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

    pub fn get_trap_cx(&self) -> &TrapContext {
        unsafe { &*self.trap_cx.get() }
    }
    pub fn get_trap_cx_mut(&self) -> &'static mut TrapContext {
        unsafe { &mut *(self.trap_cx.get() as *mut TrapContext) }
    }
    pub fn get_memory_space(&self) -> &Shared<MemorySpace> {
        unsafe { &*self.memory_space.get() }
    }
    pub fn get_memory_space_mut(&self) -> &mut Shared<MemorySpace> {
        unsafe { &mut *self.memory_space.get() }
    }

    /// 刷新TLB
    pub fn switch_pgtable(&self) {
        unsafe {
            self.get_memory_space_mut().lock().switch_page_table();
        };
    }

    // exit code
    /// 获取进程的exit code
    pub fn get_exit_code(&self) -> i32 {
        self.exit_code.load(core::sync::atomic::Ordering::SeqCst)
    }
    /// 修改进程exit code
    pub fn set_exit_code(&self, exit_code: i32) {
        self.exit_code
            .store(exit_code, core::sync::atomic::Ordering::SeqCst);
    }

    // children
    /// 添加子进程
    pub fn add_child(&self, child: Arc<TaskControlBlock>) {
        self.children.lock().insert(child.get_pid(), child);
    }
    /// 移除所有子进程
    pub fn clear_child(&self) {
        self.children.lock().clear();
    }
    /// 删除一个子线程
    pub fn remove_child(&self, pid: usize) -> Arc<TaskControlBlock> {
        // info!("[remove_child] self pid = {}", pid);
        self.children
            .lock()
            .remove(&pid)
            .expect("[remove_child] task has no such pid task")
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
        self.get_memory_space().lock().recycle_data_pages();
    }

    // fd
    /// 通过fd获取文件
    pub fn get_file_by_fd(&self, fd: usize) -> Option<Arc<dyn FileTrait>> {
        // self.fd_table.lock().get_file_by_fd(fd).unwrap_or(None)
        self.fd_table.lock().get_file_by_fd(fd).unwrap_or(None)
    }
    /// 获取当前进程的文件描述符表长度
    pub fn fd_table_len(&self) -> usize {
        self.fd_table.lock().table_len()
    }
    /// 将fd作为index获取文件描述符
    pub fn get_fd(&self, fd: usize) -> SysResult<FdInfo> {
        self.fd_table.lock().get_fd(fd)
    }
    /// 分配fd
    pub fn alloc_fd(&self, fd: FdInfo) -> SysResult<usize> {
        self.fd_table.lock().alloc_fd(fd)
    }
    /// 为以前分配了Fd的file，分配一个大于than的新fd
    pub fn alloc_fd_than(&self, fd: FdInfo, than: usize) -> usize {
        self.fd_table
            .lock()
            .alloc_fd_than(fd, than)
            .expect("task alloc fd fail")
    }
    /// 删除fd
    pub fn remove_fd(&self, fd: usize) -> SysResult {
        self.fd_table.lock().remove(fd)
    }
    /// 在指定位置设置fd
    pub fn put_fd_in(&self, fd: FdInfo, idx: usize) -> SysResult {
        self.fd_table.lock().put_in(fd, idx)
    }
    /// 清空fd_table
    pub fn clear_fd_table(&self) {
        self.fd_table.lock().clear();
    }

    pub fn detach_all_shm(&self) {
        for (_, shmid) in self.shmid_table.lock().iter() {
            SHARED_MEMORY_MANAGER
                .write()
                .get_mut(shmid)
                .unwrap()
                .detach_one(self.get_pid());
        }
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
    pub fn get_task_waker(&self) -> &Option<Waker> {
        unsafe { &*self.waker.get() }
    }
    /// 判断当前进程是否有waker
    pub fn has_waker(&self) -> bool {
        unsafe { (*self.waker.get()).is_some() }
    }
    /// 设置当前进程的waker
    pub fn set_task_waker(&self, waker: Waker) {
        unsafe { *self.waker.get() = Some(waker) }
    }

    // tid
    pub fn get_child_cleartid(&self) -> Option<usize> {
        unsafe { *self.clear_child_tid.get() }
    }
    /// 设置clear_child_tid，参考：Linux系统编程手册500页
    pub fn set_child_cleartid(&self, ctid: usize) {
        unsafe { *self.clear_child_tid.get() = Some(ctid) };
    }
    /// 设置set_child_tid字段
    pub fn set_child_settid(&self, ctid: usize) {
        unsafe { *self.set_child_tid.get() = Some(ctid) };
    }

    /// 获取当前进程所有线程的cpu时间总和
    pub fn process_cputime(&self) -> Duration {
        let mut utime = Duration::ZERO;
        let mut stime = Duration::ZERO;
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            if let Some(thread) = thread.upgrade() {
                let (ut, st) = thread.get_time_data().get_ustime();
                utime += ut;
                stime += st;
            }
        }
        utime + stime
    }

    pub fn process_ustime(&self) -> (Duration, Duration) {
        let mut utime = Duration::ZERO;
        let mut stime = Duration::ZERO;
        for (_, thread) in self.thread_group.lock().tasks.iter() {
            if let Some(thread) = thread.upgrade() {
                let (ut, st) = thread.get_time_data().get_ustime();
                utime += ut;
                stime += st;
            }
        }
        (utime, stime)
    }

    pub fn with_mut_memory_space<T>(&self, f: impl FnOnce(&mut MemorySpace) -> T) -> T {
        f(&mut self.get_memory_space().lock())
    }

    pub fn with_mut_shmid_table<T>(&self, f: impl FnOnce(&mut ShmidTable) -> T) -> T {
        f(&mut self.shmid_table.lock())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Stopped,
    Zombie,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let status = match self {
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Zombie => "Zombie",
        };
        write!(f, "{}", status)
    }
}
impl Debug for TaskStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let status = match self {
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Zombie => "Zombie",
        };
        write!(f, "{}", status)
    }
}
