#import "template.typ": img, tbl

Phoenix 是 Rust 编写的基于 RISC-V 的宏内核操作系统，结合 Rust 语言异步机制，采用无栈协程架构，支持多核运行，使用模块化的方式组织内核代码，致力于为用户程序提供完善的系统调用功能和高效的内核性能。

于 2024-07-31 结束决赛第一阶段时，截止到23点半，通过了除ltp部分测试外的所有测试点，所属 VisionFive 2 赛道排名为第2，排行榜如下图所示：

#figure(
    image("./assets/leaderboard-final.png"),
)<leaderboard-final>

于 2024-05-31 结束初赛阶段时，满分通过所有初赛测试用例。
所属 VisionFive 2 赛道的排行榜如下图所示：

#figure(
    image("./assets/leaderboard-pre.png"),
)<leaderboard>

Phoenix 各个模块完成情况如下表：

#tbl(
    table(
        columns: (20%, auto),
        inset: 12pt,
        align: horizon,
        [模块], [完成情况],
        [无栈协程], [基于全局队列实现的调度器，完善的辅助 Future 支持，支持内核态抢占式调度],
        [进程管理], [统一的进程线程抽象，可以细粒度划分进程共享的资源，支持多核运行],
        [内存管理], [实现基本的内存管理功能。使用懒分配和 Copy-on-Write 优化策略],
        [文件系统], [基于 Linux 设计的虚拟文件系统。实现页缓存加速文件读写，实现 Dentry 缓存加速路径查找，统一了页缓存与块缓存。使用开源 `rust-fatfs`库提供对 FAT32 文件系统的支持，使用`lwext4-rust`库提供对Ext4文件系统的支持],
        [信号机制], [支持用户自定义信号处理例程，有完善的信号系统，与内核其他异步设施无缝衔接],
        [设备驱动], [实现设备树解析，实现PLIC，支持异步外设中断，实现异步串口驱动],
        [网络模块], [支持Udp和Tcp套接字，Ipv4与Ipv6协议，实现异步唤醒机制]
    ),
    caption: "模块完成情况"
)
