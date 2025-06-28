#import "../template.typ": img

= 概述

== 项目介绍

Del0n1x 是一个使用 Rust 语言编写的同时适配 RISC-V64 和 LoongArch64 的宏内核，支持多核运行与无栈协程进程调度，拥有完善的内存管理和信号传递机制。在软硬件交换层面，我们实现了自己的 HAL 层，
统一调用接口，能够同时支持 RISC-V64 和 LoongArch64 指令架构。

Del0n1x 致力于实现高效清晰的代码逻辑，遵守 System Manual 手册，实现 105 个相关系统调用，并且对于其中大部分系统调用做了相对完善的错误检查和处理机制，这为我们后来适配ltp带来了便利。

Del0n1x 初赛阶段的内核主要模块和完成情况如下表格：

#table(
  columns: (auto, auto),
  align: (left, left),
  [*模块*],  [*完成情况*],
  [HAL 模块],
  [
    实现自己的HAL代码库，支持riscv64和loongarch64双架构
  ],
  [进程管理],
  [
    无栈协程调度，支持全局统一的executor调度器
    实现多线程的资源回收
    统一进程和线程的数据结构
  ],
  [文件系统], 
  [
    实现dentry构建目录树
    实现页缓存和dentry缓存加快读写

  ],
  [内存管理], 
  [
    实现COW、懒加载内存优化
    实现地址空间共享
  ],
  [时钟模块], 
  [
    实现时间轮混合最小堆的数据结构管理方式
    支持定时器唤醒机制
  ],
  [信号系统], 
  [
    支持处理用户自定义信号和sigreturn机制
  ],
  [多核管理], 
  [
    支持多核运行，但是还有部分bug
  ],
  [网络模块], 
  [
    初步完成网络模块相关代码，由于时间原因还没有适配通过网络测例
  ],
)


// TODO: 放排名截图

// TODO: 修改架构图片
#img(
  image("../assets/phoenix-design.png", width: 70%),
  caption: "Del0n1x内核架构设计"
)<phoenix-design>
