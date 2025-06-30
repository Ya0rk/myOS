#import "../components/prelude.typ": *
= 概述

== 项目介绍

Del0n1x 是一个使用 Rust 语言编写的同时适配 RISC-V64 和 LoongArch64 的宏内核，支持多核运行与无栈协程进程调度，拥有完善的内存管理和信号传递机制。在软硬件交换层面，我们实现了自己的 HAL 层，
统一调用接口，能够同时支持 RISC-V64 和 LoongArch64 指令架构。

Del0n1x 致力于实现高效清晰的代码逻辑，遵守 System Manual 手册，实现 105 个相关系统调用，并且对于其中大部分系统调用做了相对完善的错误检查和处理机制，这为我们后来适配ltp带来了便利。

Del0n1x 初赛阶段的内核主要模块和完成情况如下表格：

#table(
  columns: (2.5cm, 12cm),  // 固定列宽，第一列1.2cm，第二列12cm
  align: (left, left),
  [*模块*],  [*完成情况*],
  [HAL 模块],
  [
    实现自己的HAL代码库，支持 RISC-V64 和 LoongArch64 双架构
  ],
  [进程管理],
  [
    无栈协程调度，支持全局统一的executor调度器；
    实现多线程的资源回收；
    统一进程和线程的数据结构
  ],
  [文件系统], 
  [
    实现dentry构建目录树；
    实现页缓存和dentry缓存加快读写
  ],
  [内存管理], 
  [
    实现基本的内存管理功能；
    实现CoW、懒分配内存优化
  ],
  [时钟模块], 
  [
    实现时间轮混合最小堆的数据结构管理方式；
    支持定时器唤醒机制
  ],
  [IPC 系统], 
  [
    支持处理用户自定义信号和sigreturn机制；
    实现支持读者写者同步的管道机制；
    支持 System V 共享内存
  ],
  [网络模块], 
  [
    初步完成网络模块相关代码，由于时间原因还没有适配通过网络测例
  ],
)


// TODO: 放排名截图

// TODO: 修改架构图片

#figure(
  image("assets/整体架构图.png"),
  caption: [Del0n1x整体架构图],
  supplement: [图],
)<Del0n1x整体架构图>

#v(1em)

#h(2em)截至6月30日初赛结束前，Del0n1x的排名如下图：

#figure(
  image("assets/初赛排名.png"),
  caption: [初赛排名],
  supplement: [图]
)

#pagebreak()

整个项目的代码结构如下：

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
│   ├── net                 # 网络模块
│   ├── signal              # 信号模块
│   ├── task                # 任务控制块，任务调度
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




#pagebreak()  // 强制分页