```
/musl/ # ./run-static.sh 
[INFO][HARTID0][TASK4][sys_openat] start
[INFO][HARTID0][TASK4][sys_openat] path = /.hush_history, flags = OpenFlags(O_WRONLY | O_CREAT | O_APPEND | O_LARGEFILE)
[INFO][HARTID0][TASK4][fs_open] cwd = /musl/, path = /.hush_history, flags = OpenFlags(O_WRONLY | O_CREAT | O_APPEND | O_LARGEFILE)
[INFO][HARTID0][TASK4][do_create] start /.hush_history
[INFO][HARTID0][TASK4][do_create] succe /.hush_history
[INFO][HARTID0][TASK4][sys_openat] taskid = 4, alloc fd finished, new fd = 4
[INFO][HARTID0][TASK4][sys_lseek] start
[INFO][HARTID0][TASK4][sys_close] start, pid = 4, closed fd = 4
[INFO][HARTID0][TASK4][sys_sigaction] start signum: 28, act:4303353808, old_act: 0
[INFO][HARTID0][TASK4][sys_sigaction] taskid = 4, sa_handler = 0x1
[INFO][HARTID0][TASK4][sys_sigprocmask] start
[INFO][HARTID0][TASK4][sys_sigprocmask] taskid = 4 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK4][sys_sigprocmask] start
[INFO][HARTID0][TASK4][sys_sigprocmask] taskid = 4 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK4][sys_clone] start
[INFO][HARTID0][TASK4][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK4][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK4][process_fork] sigchld false
[INFO][HARTID0][TASK4][process_fork] self memoty space.
[INFO][HARTID0][TASK4][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK4]process fork success, new pid = 5, parent pid = 4
[INFO][HARTID0][TASK4][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353840, tls: 8, ctid: 0x400001488
[INFO][HARTID0][TASK4][sys_clone] father proc return: 5
[INFO][HARTID0][TASK4][sys_sigprocmask] start
[INFO][HARTID0][TASK4][sys_sigprocmask] taskid = 4 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK4][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91394
[INFO][HARTID0][TASK4][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK4][sys_sigprocmask] start
[INFO][HARTID0][TASK4][sys_sigprocmask] taskid = 4 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK4][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91394
[INFO][HARTID0][TASK4][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x90dfb
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_set_tid_address] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x90dfb
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x90dfb
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x90dfb
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x90dfb
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK4][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x91394
[INFO][HARTID0][TASK4][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK4][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x91394
[INFO][HARTID0][TASK4][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK4][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x91394
[INFO][HARTID0][TASK4][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK4][sys_wait4] start
[INFO][HARTID0][TASK4]wait any child
[INFO][HARTID0][TASK4][sys_wait4] current task pid = 4
[INFO][HARTID0][TASK5][sys_getpid] start
[INFO][HARTID0][TASK5][sys_setpgid] start pid: 0 pgid: 5
[INFO][HARTID0][TASK5][sys_setpgid] pid is 5 old_pgid is 4
[INFO][HARTID0][TASK5][sys_sigaction] start signum: 20, act:4303353648, old_act: 4303353680
[INFO][HARTID0][TASK5][sys_sigaction] taskid = 5, sa_handler = 0x0
[INFO][HARTID0][TASK5][sys_sigaction] start signum: 21, act:4303353648, old_act: 4303353680
[INFO][HARTID0][TASK5][sys_sigaction] taskid = 5, sa_handler = 0x0
[INFO][HARTID0][TASK5][sys_sigaction] start signum: 22, act:4303353648, old_act: 4303353680
[INFO][HARTID0][TASK5][sys_sigaction] taskid = 5, sa_handler = 0x0
[INFO][HARTID0][TASK5][sys_execve]: path: "./run-static.sh", cwd: "/musl/"
[INFO][HARTID0][TASK5][sys_exec] path = /musl//busybox, argv = ["/musl//busybox", "sh", "./run-static.sh"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK5][fs_open] cwd = /musl/, path = /musl//busybox, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK5]execve start
[INFO][HARTID0][TASK5][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][sys_set_tid_address] start
[INFO][HARTID0][TASK5][sys_getuid]: 0
[INFO][HARTID0][TASK5][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][sys_getpid] start
[INFO][HARTID0][TASK5][sys_getppid] start
[INFO][HARTID0][TASK5][sys_getcwd] start
[INFO][HARTID0][TASK5][sys_getcwd] cwd is /musl/
[INFO][HARTID0][TASK5][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][sys_uname] start
[INFO][HARTID0][TASK5][sys_openat] start
[INFO][HARTID0][TASK5][sys_openat] path = ./run-static.sh, flags = OpenFlags(O_CLOEXEC | O_LARGEFILE)
[INFO][HARTID0][TASK5][fs_open] cwd = /musl/, path = /musl/run-static.sh, flags = OpenFlags(O_CLOEXEC | O_LARGEFILE)
[INFO][HARTID0][TASK5][sys_openat] taskid = 5, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK5][sys_fcntl] start, fd = 3, cmd = FcntlFlags(F_SETFD)
[INFO][HARTID0][TASK5][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK5][sys_sigaction] start signum: 3, act:4303354144, old_act: 4303354176
[INFO][HARTID0][TASK5][sys_sigaction] taskid = 5, sa_handler = 0x1200c3030
[INFO][HARTID0][TASK5][sys_sigaction] start signum: 17, act:4303354144, old_act: 4303354176
[INFO][HARTID0][TASK5][sys_sigaction] taskid = 5, sa_handler = 0x1200c3030
[INFO][HARTID0][TASK5]read file: /musl/run-static.sh, offset: 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913a1
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913a1
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913a1
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913a1
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913a1
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "argv"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe argv ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913a4
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913a4
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913cf
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "argv"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913cf
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe argv ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d3
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913d3
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913d3
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913d3
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913d3
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "basename"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe basename ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91473
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91475
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91475
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "basename"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91473
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe basename ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "clocale_mbfuncs"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe clocale_mbfuncs ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146e
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ba
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913ba
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "clocale_mbfuncs"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9146e
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe clocale_mbfuncs ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d7
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913d7
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913d7
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913d7
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913d7
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "clock_gettime"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe clock_gettime ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91473
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91473
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "clock_gettime"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9146f
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe clock_gettime ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "dirname"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe dirname ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146e
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9146e
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "dirname"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d8
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d8
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe dirname ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ba
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ba
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ba
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ba
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ba
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "env"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354640, old_act: 4303354672
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe env ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354736, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d2
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d2
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc90
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc90, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "env"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913b8
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913b8
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91475
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe env ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91478
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x91478
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x91478
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91478
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91478
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "fdopen"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe fdopen ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913cf
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d5
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d5
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "fdopen"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/testsuite-ckFiBJ, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/testsuite-ckFiBJ, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/testsuite-ckFiBJ
[INFO][HARTID0][TASK7][do_create] succe /tmp/testsuite-ckFiBJ
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913a8
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913a8
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/testsuite-ckFiBJ, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/testsuite-ckFiBJ, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/testsuite-ckFiBJ, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913cf
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe fdopen ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913a4
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913a4
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913a4
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913a4
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913a4
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "fnmatch"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe fnmatch ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913bd
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913bd
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "fnmatch"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe fnmatch ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "fscanf"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe fscanf ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91475
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91475
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "fscanf"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9147c
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_pipe] start!
[INFO][HARTID0][TASK7][sys_pipe] taskid = 7, alloc read_fd = 3, write_fd = 4
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][getname] this is pipe file
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 4
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_LdmCdm, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_LdmCdm, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_LdmCdm
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_LdmCdm
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_LdmCdm, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_LdmCdm, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_LdmCdm, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_LdmCdm, offset: 8
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_MkLcPD, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_MkLcPD, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_MkLcPD
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_MkLcPD
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_MkLcPD, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_MkLcPD, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_MkLcPD, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_EbgALI, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_EbgALI, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_EbgALI
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_EbgALI
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_EbgALI, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_EbgALI, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_EbgALI, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_EbgALI, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_EbgALI, offset: 7
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_mdhnbN, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_mdhnbN, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_mdhnbN
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_mdhnbN
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_mdhnbN, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_mdhnbN, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_mdhnbN, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_mdhnbN, offset: 13
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_eOgJJc, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_eOgJJc, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_eOgJJc
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_eOgJJc
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_eOgJJc, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_eOgJJc, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9146d
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_eOgJJc, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9147c
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe fscanf ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9147b
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x9147b
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x9147b
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x9147b
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9147b
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "fwscanf"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe fwscanf ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146e
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9146e
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "fwscanf"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_fjbgmN, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_fjbgmN, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_fjbgmN
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_fjbgmN
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_fjbgmN, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_fjbgmN, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_fjbgmN, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_fjbgmN, offset: 8
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_FChnLe, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_FChnLe, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_FChnLe
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_FChnLe
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_FChnLe, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_FChnLe, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_FChnLe, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_npEEFj, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_npEEFj, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_npEEFj
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_npEEFj
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_npEEFj, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_npEEFj, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_npEEFj, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_npEEFj, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_npEEFj, offset: 7
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_gOjMCB, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_gOjMCB, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_gOjMCB
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_gOjMCB
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_gOjMCB, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_gOjMCB, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_gOjMCB, offset: 0
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_gOjMCB, offset: 13
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_clock_gettime] start, clock id = 0
[INFO][HARTID0][TASK7][sys_clock_gettime] finish
[INFO][HARTID0][TASK7][sys_openat] start
[INFO][HARTID0][TASK7][sys_openat] path = /tmp/tmpfile_ggLgiF, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_ggLgiF, flags = OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE)
[INFO][HARTID0][TASK7][do_create] start /tmp/tmpfile_ggLgiF
[INFO][HARTID0][TASK7][do_create] succe /tmp/tmpfile_ggLgiF
[INFO][HARTID0][TASK7][sys_openat] taskid = 7, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK7][sys_unlinkat] start fd: -100, base: /musl/, path: /tmp/tmpfile_ggLgiF, flags: 0
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = /tmp/tmpfile_ggLgiF, flags = OpenFlags(O_RDWR)
[INFO][HARTID0][TASK7][sys_unlink] finished
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7]read file: /tmp/tmpfile_ggLgiF, offset: 0
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_lseek] start
[INFO][HARTID0][TASK7][sys_close] start, pid = 7, closed fd = 3
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91472
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe fwscanf ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "iconv_open"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe iconv_open ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91477
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d8
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d8
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "iconv_open"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91477
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe iconv_open ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913b5
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "inet_pton"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
========== START entry-static.exe inet_pton ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9146f
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x9146f
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "inet_pton"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913bd
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe inet_pton ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91470
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x91470
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x91470
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91470
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91470
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "mbc"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354640, old_act: 4303354672
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe mbc ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354736, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d3
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d3
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc90
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc90, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "mbc"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d2
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bb
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d2
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe mbc ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x9147a
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x9147a
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x9147a
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x9147a
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x9147a
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "memstream"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354624, old_act: 4303354656
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe memstream ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354720, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x91477
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x91477
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc80
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc80, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "memstream"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x90870
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_mmap] addr:0x40001000, length:0x1000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x1000, prot:MmapProt(PROT_READ | PROT_WRITE), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x91473
[INFO][HARTID0][TASK7][sys_exit_group] start, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] start, taskid = 7, exitcode = 0
[INFO][HARTID0][TASK7][sys_exit_group] task exitcode = 0
[INFO][HARTID0][TASK7][do_exit] Task pid = 7 exit;
[INFO][HARTID0][TASK7][handle exit] clear child tid 0x1200a27c8
[INFO][HARTID0][TASK7][do_exit] task to info parent pid = 6, exit code = 0
[INFO][HARTID0][TASK6][sys_wait4]: task 6 find a child: pid = 7, exit_code = 0 , exitcode << 8 = 0.
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x90870
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
Pass!
========== END entry-static.exe memstream ==========
[INFO][HARTID0][TASK6][sys_exit_group] start, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] start, taskid = 6, exitcode = 1
[INFO][HARTID0][TASK6][sys_exit_group] task exitcode = 256
[INFO][HARTID0][TASK6][do_exit] Task pid = 6 exit;
[INFO][HARTID0][TASK6][handle exit] clear child tid 0x120014920
[INFO][HARTID0][TASK6][do_exit] task to info parent pid = 5, exit code = 256
[INFO][HARTID0][TASK5][sys_wait4]: task 5 find a child: pid = 6, exit_code = 256 , exitcode << 8 = 0.
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400003000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK5][sys_clone] start
[INFO][HARTID0][TASK5][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK5][process_fork] sigchld false
[INFO][HARTID0][TASK5][process_fork] self memoty space.
[INFO][HARTID0][TASK5][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK5]process fork success, new pid = 6, parent pid = 5
[INFO][HARTID0][TASK5][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303353664, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK5][sys_clone] father proc return: 6
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_sigprocmask] start
[INFO][HARTID0][TASK5][sys_sigprocmask] taskid = 5 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x40002000..VA:0x40003000, map_perm: MapPerm(R | W | U), vma_type: Heap }, VPN:0x40002 at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913ab
[INFO][HARTID0][TASK5][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK5][sys_wait4] start
[INFO][HARTID0][TASK5]wait any child
[INFO][HARTID0][TASK5][sys_wait4] current task pid = 5
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913bc
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fd at page table PPN:0x913bc
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x1201fb000..VA:0x1201fe000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x1201fc at page table PPN:0x913bc
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x0, set = SigMask(0x0), how = 2
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400001000..VA:0x400002000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400001 at page table PPN:0x913bc
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x400000000..VA:0x400001000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400000 at page table PPN:0x913bc
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK6][sys_execve]: path: "./runtest.exe", cwd: "/musl/"
[INFO][HARTID0][TASK6][sys_exec] path = ./runtest.exe, argv = ["./runtest.exe", "-w", "entry-static.exe", "pthread_cancel_points"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK6][fs_open] cwd = /musl/, path = ./runtest.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK6]execve start
[INFO][HARTID0][TASK6][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][sys_set_tid_address] start
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK6][sys_sigaction] start signum: 17, act:4303354608, old_act: 4303354640
[INFO][HARTID0][TASK6][sys_sigaction] taskid = 6, sa_handler = 0x120000634
========== START entry-static.exe pthread_cancel_points ==========
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK6][sys_clone] start
[INFO][HARTID0][TASK6][sys_clone] start child_stack 0, flag: CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] start, flags = CloneFlags(SIGCHLD)
[INFO][HARTID0][TASK6][process_fork] sigchld false
[INFO][HARTID0][TASK6][process_fork] self memoty space.
[INFO][HARTID0][TASK6][from_user_lazily] enter during process fork
[INFO][HARTID0][TASK6]process fork success, new pid = 7, parent pid = 6
[INFO][HARTID0][TASK6][sys_clone] start, flags: CloneFlags(SIGCHLD), ptid: 4303354704, tls: 8, ctid: 0x0
[INFO][HARTID0][TASK6][sys_clone] father proc return: 7
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913d2
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x120013000..VA:0x120015000, map_perm: MapPerm(R | W | U), vma_type: Elf }, VPN:0x120014 at page table PPN:0x913d2
[INFO][HARTID0][TASK7][handle_page_fault] page cnt:2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7ffdfeff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 2
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x0, old_limit = 0x1007ffc70
[INFO][HARTID0][TASK7][sys_prlimit64] start, pid = 0, resource = 3, new_limit = 0x1007ffc70, old_limit = 0x0
[INFO][HARTID0][TASK7][sys_execve]: path: "entry-static.exe", cwd: "/musl/"
[INFO][HARTID0][TASK7][sys_exec] path = entry-static.exe, argv = ["entry-static.exe", "pthread_cancel_points"], env = ["PATH=/bin:/", "HOME=/", "LD_LIBRARY_PATH=/", "TERM=screen", "PWD=/musl/", "HUSH_VERSION=1.33.1"]
[INFO][HARTID0][TASK7][fs_open] cwd = /musl/, path = entry-static.exe, flags = OpenFlags(0x0)
[INFO][HARTID0][TASK7]execve start
[INFO][HARTID0][TASK7][init_stack] in with sp:0x1007ffff0
[INFO][HARTID0][TASK6][VmArea::handle_page_fault] VmArea { range_va: VA:0x100000000..VA:0x100800000, map_perm: MapPerm(R | W | U), vma_type: Stack }, VPN:0x1007ff at page table PPN:0x913af
[INFO][HARTID0][TASK6][handle_page_fault] page cnt:1
[INFO][HARTID0][TASK6][sys_sigprocmask] start
[INFO][HARTID0][TASK6][sys_sigprocmask] taskid = 6 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK6][sys_sigtimedwait] start
[INFO][HARTID0][TASK6][sys_wait4] start
[INFO][HARTID0][TASK6]wait target pid = 7
[INFO][HARTID0][TASK6][sys_wait4] current task pid = 6
[INFO][HARTID0][TASK7][sys_set_tid_address] start
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x300000000, set = SigMask(SIGCANCEL | SIGSYNCCALL), how = 1
[INFO][HARTID0][TASK7][sys_mmap] addr:0x0, length:0x23000, prot:MmapProt(0x0), flags:MmapFlags(MAP_PRIVATE | MAP_ANONYMOUS), fd:18446744073709551615, offset:0x0
[INFO][HARTID0][TASK7][VmArea::handle_page_fault] VmArea { range_va: VA:0x400002000..VA:0x400023000, map_perm: MapPerm(R | W | U), vma_type: Mmap }, VPN:0x400022 at page table PPN:0x91476
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xfffffffc7fffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK7][sys_clone] start
[INFO][HARTID0][TASK7][sys_clone] start child_stack 17180011216, flag: CloneFlags(CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD | CLONE_SYSVSEM | CLONE_SETTLS | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID | CLONE_DETACHED)
[INFO][HARTID0][TASK7][thread_fork] start, flags = CloneFlags(CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD | CLONE_SYSVSEM | CLONE_SETTLS | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID | CLONE_DETACHED)
[INFO][HARTID0][TASK7][thread_fork] sigchld
[INFO][HARTID0][TASK7][fork]: child thread tid 8, parent process pid 7
[INFO][HARTID0][TASK7][sys_clone] start, flags: CloneFlags(CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD | CLONE_SYSVSEM | CLONE_SETTLS | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID | CLONE_DETACHED), ptid: 17180011304, tls: 4832503752, ctid: 0x400022bd0
[INFO][HARTID0][TASK7][sys_clone] father proc return: 8
[INFO][HARTID0][TASK8][sys_sigprocmask] start
[INFO][HARTID0][TASK8][sys_sigprocmask] taskid = 8 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK8][sys_futex] start, futex_op = FutexOp(FUTEX_PRIVATE)
[INFO][HARTID0][TASK8][sys_futex] use_op = FutexOp(0x0)
[INFO][HARTID0][TASK8][sys_futex] wait, uaddr = 0x12009dc08, taskid = 8, now_val = 2147483648, val = 2147483648
[INFO][HARTID0][TASK8][sys_futex] wait, now_val = 2147483648, val = 2147483648, timeout = 0
[INFO][HARTID0][TASK8][do_futex_wait] uaddr = 0x12009dc08
[INFO][HARTID0][TASK8][futex_future] poll pid = 8, uaddr = 0x12009dc08, is_register = false
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_sigaction] start signum: 33, act:4303354944, old_act: 0
[INFO][HARTID0][TASK7][sys_sigaction] taskid = 7, sa_handler = 0x12003233c
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0xffffffffffffffff, set = SigMask(SIGHUP | SIGINT | SIGQUIT | SIGILL | SIGTRAP | SIGABRT | SIGBUS | SIGFPE | SIGKILL | SIGUSR1 | SIGSEGV | SIGUSR2 | SIGPIPE | SIGALRM | SIGTERM | SIGSTKFLT | SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU | SIGURG | SIGXCPU | SIGXFSZ | SIGVTALRM | SIGPROF | SIGWINCH | SIGIO | SIGPWR | SIGSYS | SIGTIMER | SIGCANCEL | SIGSYNCCALL | SIGRT_3 | SIGRT_4 | SIGRT_5 | SIGRT_6 | SIGRT_7 | SIGRT_8 | SIGRT_9 | SIGRT_10 | SIGRT_11 | SIGRT_12 | SIGRT_13 | SIGRT_14 | SIGRT_15 | SIGRT_16 | SIGRT_17 | SIGRT_18 | SIGRT_19 | SIGRT_20 | SIGRT_21 | SIGRT_22 | SIGRT_23 | SIGRT_24 | SIGRT_25 | SIGRT_26 | SIGRT_27 | SIGRT_28 | SIGRT_29 | SIGRT_30 | SIGRT_31 | SIGMAX), how = 0
[INFO][HARTID0][TASK7][sys_tkill] start, tid = 8, sig = 33
[INFO][HARTID0][TASK7][sys_sigprocmask] start
[INFO][HARTID0][TASK7][sys_sigprocmask] taskid = 7 ,set = 0x10000, set = SigMask(SIGCHLD), how = 2
[INFO][HARTID0][TASK7][sys_futex] start, futex_op = FutexOp(FUTEX_WAKE | FUTEX_PRIVATE)
[INFO][HARTID0][TASK7][sys_futex] use_op = FutexOp(FUTEX_WAKE)
[INFO][HARTID0][TASK7][sys_futex] wake, val = 2147483647
[INFO][HARTID0][TASK8][futex_future] poll pid = 8, uaddr = 0x12009dc08, is_register = true
[INFO][HARTID0][TASK8][futex queue] remove pid = 8
[INFO][HARTID0][TASK8][do_signal] task id = 8, find a signal: 33, handler = 0x12003233c, flags = SigActionFlag(SA_SIGINFO | SA_ONSTACK | SA_RESTART).
[INFO][HARTID0][TASK8][do_signal] default stack
[INFO][HARTID0][TASK8][do_signal] before flash
[INFO][HARTID0][TASK8][flash] in with args: handler:0x12003233c, new_sp:0x400022590, sigret:0x9000000000201000, signo:33
[INFO][HARTID0][TASK8][sys_sigreturn] start
[INFO][HARTID0][TASK8][sys_futex] start, futex_op = FutexOp(FUTEX_PRIVATE)
[INFO][HARTID0][TASK8][sys_futex] use_op = FutexOp(0x0)
[INFO][HARTID0][TASK8][sys_futex] wait, uaddr = 0x12009dbe0, taskid = 8, now_val = 2147483648, val = 2147483648
[INFO][HARTID0][TASK8][sys_futex] wait, now_val = 2147483648, val = 2147483648, timeout = 0
[INFO][HARTID0][TASK8][do_futex_wait] uaddr = 0x12009dbe0
[INFO][HARTID0][TASK8][futex_future] poll pid = 8, uaddr = 0x12009dbe0, is_register = false
[INFO][HARTID0][TASK7][sys_futex] start, futex_op = FutexOp(FUTEX_PRIVATE)
[INFO][HARTID0][TASK7][sys_futex] use_op = FutexOp(0x0)
[INFO][HARTID0][TASK7][sys_futex] wait, uaddr = 0x400022b30, taskid = 7, now_val = 2, val = 2
[INFO][HARTID0][TASK7][sys_futex] wait, now_val = 2, val = 2, timeout = 0
[INFO][HARTID0][TASK7][do_futex_wait] uaddr = 0x400022b30
[INFO][HARTID0][TASK7][futex_future] poll pid = 7, uaddr = 0x400022b30, is_register = false
QEMU: Terminated

```