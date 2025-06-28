# Del0n1x

![](./report/iamge/school.jpg)

## 项目简介

[github仓库链接](https://github.com/Ya0rk/myOS): 欢迎向我们提出issue.

Del0n1x 是一个使用 Rust 语言编写的同时适配 RISC-V64 和 LoongArch64 的多核操作系统，目标是实现一个 Linux 兼容的多核操作系统，支持进程调度、文件系统、网络等功能。

![](./整体架构图.png)

## 运行项目

进入`cd os`文件，然后执行`make run ARCH=xxx OVERWRITE=true`

指令说明：

- 第一次运行需要在项目根目录准备`sdcard-rv.img`和`sdcard-la.img`(可以从官方的github测试仓库下载并编译)
- `ARCH`: 内核架构，如果不加该参数默认是`riscv`；如果需要龙芯，则：`ARCH = loongarch64`
- `OVERWRITE`：因为我的os实现了真正的向镜像img读和写，为了避免破坏镜像，该参数回复制一份img用于挂载


## 项目调试

#### riscv64

进入`cd os`文件，然后执行`make gdbserver`，另外打开一个终端，执行`make gdbclient`.

#### loongarch64

进入`cd os`文件，然后执行`make gdbserver ARCH=loongarch64`，另外打开一个终端，执行`make LAgdbclient`.

## 目录架构

```
os
├── linker                  # 程序链接脚本
├── src
│   ├── arch                # 架构相关的汇编
│   ├── driver              # 块设备驱动
│   ├── fuse                # 文件系统
│   ├── hal                 # 架构相关代码
│   ├── ipc                 # 进程间通信相关的部分代码
│   ├── mm                  # 内存页表
│   ├── net                 # 网络
│   ├── signal              # 信号
│   ├── task                # TCB
│   ├── utils               # 一些工具
│   ├── sync                # 同步相关
│   ├── syscall             # 系统调用
│   ├── entry_la.asm        # 龙芯入口初始化汇编函数
│   ├── entry.asm           # riscv入口初始化函数
│   ├── console.rs
│   ├── lang_items.rs
│   ├── Makefile
│   └── main.rs
user                        # 用户程序
├── src
│   ├── bin
│   │   ├── autorun.rs      # 自动测试
│   │   ├── gbshell.rs      # glibc的busybox shell
│   │   ├── huge_write.rs   # 测试文件系统写入速度
│   │   ├── initproc.rs     # 调用user_shell，进入自己实现的终端
│   │   ├── mbshell.rs      # musl的busybox shell
│   │   └── user_shell.rs
vendor                      # 第三方依赖
report                      # 文档
bootloader                  # 引导加载程序
```

## 项目成员

- 姚俊杰(345024941@qq.com): 进程模块、信号模块、网络模块等
- 林顺喆(): 内存模块、龙芯等
- 卢家鼎(1277319667@qq.com): 文件系统、块设备等
- 指导老师：夏文  仇洁婷

## 工具链

`cargo`:

```
❯ cargo version
cargo 1.86.0-nightly (088d49608 2025-01-10)2025-01-10
```

`Rust工具链`:

```
loongarch64-unknown-linux-gnu
loongarch64-unknown-none
riscv64gc-unknown-linux-musl
riscv64gc-unknown-none-elf
```

`qemu`:

```
❯ qemu-system-riscv64 --version
QEMU emulator version 9.2.1
Copyright (c) 2003-2024 Fabrice Bellard and the QEMU Project developers
❯ qemu-system-loongarch64 --version
QEMU emulator version 9.2.1
Copyright (c) 2003-2024 Fabrice Bellard and the QEMU Project developers
```

`riscv64-unknown-elf-gdb`: 

```
❯ riscv64-unknown-elf-gdb --version
GNU gdb (GDB) 16.2
Copyright (C) 2024 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
```

`loongarch64-unknown-linux-gnu-gdb`:

```
❯ loongarch64-unknown-linux-gnu-gdb --version
GNU gdb (GDB) 16.2
Copyright (C) 2024 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
```

其余工具链可以通过[官方链接](https://github.com/oscomp/testsuits-for-oskernel/tree/pre-2025?tab=readme-ov-file)中的Dockerfile下载.