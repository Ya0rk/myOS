#import "../components/prelude.typ": *
= 设备

== 设备管理模块概述

设备的管理在操作系统中起到至关重要的部分。Del0n1x 对设备进行抽象和封装，确保内核对设备进行规范的管理，向其他模块提供简便易用的接口，并为拓展新的设备提供方便。

设备管理模块实现了设备的发现和初始化、驱动程序匹配与加载等功能。操作系统依赖设备管理模块与计算机系统中的硬件部分进行信息的交换，设备管理模块的设计直接影响整个系统的可用性、稳定性与性能。


设备管理模块的核心功能之一是向文件系统层提供字符设备、块设备两种类型设备的访问接口。Del0n1x 定义了字符设备和块设备的统一抽象`Device trait`，并在此基础上分别拓展了字符设备和块设备的抽象：`BlockDevice trait`，和 `CharDevice trait`，用于承载两种设备的专有操作。

#code-figure(
```rust
pub trait BlockDevice: Device {
    fn num_blocks(&self) -> usize;
    fn block_size(&self) -> usize;
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult;
    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult;
    fn flush(&self) -> DevResult;
}
```  
,  
    caption: [BlockDevice trait定义],
    label-name: "block-device trait"
)


Del0n1x 在设备管理模块中创建了一个全局的设备管理器`DeviceManager`，这个结构不仅能在初始化阶段对 UART0 串口、中断控制器等核心设备进行初始化，还负责为物理设备驱动提供注册新设备的接口。同时，该结构维护了设备号到`dyn Device`实例的映射，为`devfs`绑定设备提供了方便。


// #figure(
//   image("img/rCore换栈.png"),
//   caption: [rCore 上下文切换],
//   supplement: [图],
// )<rCore上下文切换>

== 设备树

操作系统通过解析设备树实现对设备的发现。Del0n1x 内核获取到设备树的地址的流程如下：

#enum(
    enum.item(1)[OpenSBI启动：当系统启动时，OpenSBI固件首先运行。它完成基础的硬件初始化，如内存控制器设置、I/O 初始化等],
    enum.item(2)[传递控制权到内核：OpenSBI初始化完成后，将控制权传递给内核的入口点，并传递必要的参数。这些参数包括：],
    indent: 2em
)
#list(
    [`hart_id`：当前硬件线程的 ID。],
    [`dtb_addr`：设备树地址，该地址指向设备树描述符（DTB），描述了系统的硬件布局和配置信息。],
    indent: 4em
)
// + OpenSBI启动：当系统启动时，OpenSBI
//   固件首先运行。它完成基础的硬件初始化，如内存控制器设置、I/O 初始化等。

// + 传递控制权到内核：OpenSBI
//   初始化完成后，将控制权传递给内核的入口点，并传递必要的参数。这些参数包括：

  // - `hart_id`：当前硬件线程的 ID。
  // - `dtb_addr`：设备树地址，该地址指向设备树描述符（DTB），描述了系统的硬件布局和配置信息。

// 对于 LoongArch QEMU virt 平台，我们通过查看 QEMU 源码确定了设备树的地址。

#h(2em)Del0n1x 要求驱动程序实现 `probe` 方法，从设备树的根节点搜索匹配的设备并解析设备树中的有关字段（MMIO 地址、中断号、设备时钟等），并依据这些信息，动态地构造并向`DeviceManager`注册内核中的设备实例，供给其他模块使用。通过设备树解析，我们可以实现同一份内核二进制在不同的硬件上启动。

== TTY 子系统

=== 总览

Del0n1x 实现了功能相对完善了 TTY 子系统，支持异步读、写串口输入输出，支持响应来自串口的外部中断。

Del0n1x 的 TTY 子系统采用三层架构：

#list(
    [顶层：`TtyStruct` ：实现`CharDevice trait`，与文件系统层对接；管理输入输出格式及行规程；],
    [中间层：`SerialDriver` ：内置`RingBuffer`，响应外部中断，负责处理异步协程的睡眠和唤醒逻辑；],
    [底层：`impl UartDriver trait` ：负责与物理串口设备的直接交互，实现从设备树自加载的功能。],
    indent: 2em
)

#h(2em)这样的分层设计方法保证了 TTY 子系统的可拓展性，为后续的应用移植和适配提供方便。

=== TTY Struct和行规程

Del0n1x 设计了一个较为完整的 TTY Struct，用于为文件系统层请求的特定格式的输入输出提供支持：
#code-figure(
```rs
pub struct TtyStruct {
    /// 对接下一层，SerialDriver实现这个trait
    pub driver: Arc<dyn TtyDriver>,
    /// 用于设置行规程、输入输出格式和设备控制信息
    pub termios: RwLock<termios::Termios>,
    /// 为 N_TTY 行规程使用，判断当前是否为行编辑模式
    pub n_tty_mode: RwLock<TtyLineDiscMode>,
    /// 存储行规程，行规程使用策略模式
    pub ldisc: SyncUnsafeCell<Arc<dyn LineDiscPolicy>>,
    /// 前台进程组
    pub fg_pgid: RwLock<u32>,
    /// 终端窗口尺寸
    pub win_size: RwLock<WinSize>,
    /// 行缓冲区
    pub lbuffer: Shared<LineBuffer>,
    /// 主设备号
    pub major: MajorNumber,
    /// 次设备号
    pub minor: usize,   
}
```   ,
    caption: [TTY Struct 类型及字段含义],
    label-name: "tty-struct"
)


Del0n1x 支持用户进程通过`ioctl`设置 `Termios`、前台进程组、终端尺寸、行规程等参数，并实现了基础的行规程功能。行规程是 TTY 子系统中用于规范底层传入字节流的组织模式的一个模块。Del0n1x 使用策略模式，支持行规程的创建和修改。

当前 Del0n1x 实现了 Linux 的 N_TTY 行规程，支持进程在非行编辑模式下获取串口的输入。

#code-figure(
```rs
#[async_trait]
pub trait LineDiscPolicy : Sync + Send + 'static {
    async fn read(&self, tty: &TtyStruct, buf: &mut [u8]) -> usize;
    async fn write(&self, tty: &TtyStruct, buf: &[u8]) -> usize;
    async fn poll_in(&self, tty: &TtyStruct) -> bool;
    async fn poll_out(&self, tty: &TtyStruct) -> bool;
    async fn set_mode(&self, tty: &TtyStruct, mode: TtyLineDiscMode);
}
```,
    caption: [行规程策略定义],
    label-name: "tty-line-discipline"
)

=== 异步串口 IO

数据驱动的异步读写机制是 IO 驱动子系统的核心。内核依赖响应外部中断实现不忙等、不空转的良好 IO 机制，极大提高CPU利用率。不同于 Linux 使用`tty_port`和`uart_port`实现的多级回调读写， Del0n1x 在`TtyStruct`层和物理驱动层之间实现了一个独立的`SerialDriver`层，用于响应中断、控制协程的调度。

#code-figure(
```rs
pub struct SerialDriver {
    pub uart: Arc<dyn UartDriver>,
    pub icbuffer: Shared<CharBuffer>,
    pub ocbuffer: Shared<CharBuffer>,
    pub read_queue: Shared<VecDeque<Waker>>,
    pub write_queue: Shared<VecDeque<Waker>>,
}
```,
    caption: [SerialDriver 中间层定义],
    label-name: "serial-driver"
)


#h(2em)`SerialDriver`负责串口设备中断的处理。以输入举例，当 CPU 收到来自串口设备的一个中断时，`SerialDriver`将输入的字符添加到`icbuffer`，并唤醒`read_queue`中的一个进程。如果当前没有进程正在等待，则输入的字符将停留在`icbuffer`缓冲区，等待被读取。当一个前台进程尝试读取串口输入时，如果`icbuffer`和物理串口设备中均没有可读取的字节，该进程会将自身的`waker`保存在`read_queue`中，随即调用`suspend_now()`让出 CPU ，并等待来自串口设备的一个中断将其唤醒。

// #h(2em)

#pagebreak()  // 强制分页