#import "../template.typ": img

= 概述

== Phoenix 介绍

Phoenix 是 Rust 编写的基于 RISC-V 的多核异步宏内核模块化操作系统，使用了以 Future 抽象为代表的无栈协程异步模型，提供统一的线程和进程表示，细粒度的资源共享以及段式地址空间管理，拥有基于 Linux 设计的完善的虚拟文件系统以及完善的信号机制。

Phoenix 使用模块化的方式组织内核代码，将宏内核各个部分独立成不同的模块。通过明确模块之间的接口，减少模块间的相互依赖和干扰，从而使内核代码复用成为可能，目标是为以后参与操作系统竞赛的同学提供功能与性能兼备的模块。

Phoenix 致力于实现高质量的代码，使内核兼具完善的功能和高效的性能。功能方面，Phoenix 通过了初赛所有系统调用测试用例，并通过了除ltp部分测试外所有测试点。性能方面，使用懒分配和写时复制机制优化 `execve` 执行速度，使用页缓存加速文件读写，实现了页缓存和块缓存的统一，减少磁盘 IO 访问次数。

Phoenix 严格遵循 System Calls Manual 手册进行开发，确保实现的所有系统调用符合 Linux raw syscall 规范，为运行在 Phoenix 的用户态程序提供了完备可靠的系统调用支持。

== Phoenix 整体架构

```
.
├── arch                    # 平台相关的包装函数与启动函数
├── config                  # 配置常量
├── crates                  # 自己编写的功能单一的库
│   ├── backtrace           # 堆栈回溯
│   ├── macro-utils         # 宏工具
│   ├── range-map           # 范围映射
│   ├── recycle-allocator   # ID回收分配器
│   ├── ring-buffer         # 环形队列缓冲区
│   └── sbi-print           # SBI打印工具
├── docs                    # 文档
├── driver                  # 驱动模块
├── kernel                  # 内核
│   ├── src
│   │   ├── ipc             # 进程间通信机制
│   │   ├── mm              # 内存管理
│   │   ├── net             # 网络
│   │   ├── processor       # 多核心管理
│   │   ├── syscall         # 系统调用
│   │   ├── task            # 进程管理
│   │   ├── trap            # 异常处理
│   │   ├── utils           # 工具
│   │   ├── boot.rs         # 内核启动通用函数
│   │   ├── impls.rs        # 模块接口实现
│   │   ├── main.rs         # 主函数
│   │   ├── panic.rs
│   │   └── trampoline.asm  # 信号跳板
│   ├── build.rs
│   ├── Cargo.toml
│   ├── linker.ld           # 链接脚本
│   └── Makefile
├── modules                 # 内核各个模块
│   ├── device-core         # 设备API
│   ├── executor            # 异步调度器
│   ├── ext4                # Ext4文件系统支持
│   ├── fat32               # FAT32文件系统支持
│   ├── logging             # 日志系统
│   ├── memory              # 基础内存模块
│   ├── net                 # 基础信号模块
│   ├── page                # 页缓存与块缓存
│   ├── signal              # 基础信号模块
│   ├── sync                # 同步原语
│   ├── systype             # 系统调用类型
│   ├── time                # 时间模块
│   ├── timer               # 定时器模块
│   ├── vfs                 # 虚拟文件系统模块
│   └── vfs-core            # 虚拟文件系统接口
├── testcase                # 测试用例
├── third-party             # 第三方库
│   └── vendor              # Rust库缓存
├── user                    # 用户程序
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── LICENSE
├── Makefile
├── README.md
├── rustfmt.toml
└── rust-toolchain.toml
```

#img(
  image("../assets/phoenix-design.png", width: 70%),
  caption: "Phoenix内核架构设计"
)<phoenix-design>
