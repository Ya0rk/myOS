# 嵌入式操作系统字符设备栈设计文档

## 1. 设计哲学与目标

本设计的核心目标是借鉴Linux的成功经验，构建一个分层、解耦、可扩展的字符设备I/O模型。即使当前只关注一个虚拟串口设备，一个良好的架构也能让未来添加新设备（如物理UART、管道、零设备等）变得简单。

**核心原则**:
1.  **分层与职责分离**: 每一层只做一件事，并做好。
2.  **面向接口编程**: 上层不关心下层的具体实现，只通过约定的接口（Traits）进行交互。
3.  **统一用户视图**: 从用户任务（进程）的角度看，所有字符设备都表现为可以通过文件描述符进行读写的文件。

## 2. 核心组件与层次关系图

我们将整个栈分为四个主要层次，外加两个基础抽象：

```
+-----------------------------------------------------------------+
|                         用户任务 (Task)                         |
|         (syscall: open, read, write, ioctl, close)              |
+--------------------------------- | -----------------------------+
                                  | (fd, path)
+--------------------------------- V -----------------------------+
|      [抽象父类] Device Inode / VFS (文件系统接口层)             |
| - 角色: 实现“一切皆文件”，管理路径和文件描述符(fd)。            |
| - 核心: `open(path)` -> `(major, minor)`, `read(fd)` -> `dev_read()` |
+--------------------------------- | -----------------------------+
                                  | (major, minor)
+--------------------------------- V -----------------------------+
|          字符设备Core (Character Device Core)                   |
| - 角色: 字符设备的“工商总局”，管理所有字符设备驱动。            |
| - 核心: `register_driver(major, driver)`, `drivers[major]->read()` |
+--------------------------------- | -----------------------------+
                                  | (minor)
+--------------------------------- V -----------------------------+
|               TTY Core (终端业务逻辑层)                         |
| - 角色: 处理终端通用逻辑，如行编辑、回显。                      |
| - 核心: `tty_receive_char()`, `tty_read()` -> `ldisc_read()`      |
+--------------------------------- | -----------------------------+
                                  |
+--------------------------------- V -----------------------------+
|               UART驱动 (UART Driver)                            |
| - 角色: 特定UART的“操作手册”，直接与硬件交互。                  |
| - 核心: `uart_isr()`, `uart_putc()` -> (写MMIO寄存器)             |
+--------------------------------- | -----------------------------+
                                  | (MMIO)
+--------------------------------- V -----------------------------+
|                      虚拟串口硬件 (MMIO)                        |
+-----------------------------------------------------------------+
```

## 3. 各层级功能分配与接口设计

### 3.1 [抽象父类] Device Inode / VFS (文件系统接口层)

这是用户任务直接交互的最高层。

**职责**:
-   维护一个从路径名（如`/dev/ttyS0`）到设备标识（主/次设备号）的映射。
-   管理文件描述符表，将`fd`映射到具体的打开文件实例。
-   提供标准的、统一的系统调用接口。

**需要Device Core/Inode抽象父类提供的接口**:
这个问题的提法可以理解为“VFS层需要下层提供什么”。VFS层需要一个通用的设备文件操作集。

**设计与接口**:
我们可以定义一个`FileOps` trait（类似Linux的`file_operations`）。

```rust
// in vfs/mod.rs
pub trait FileOps {
    // 当用户调用read(fd,...)时，VFS最终会调用这个
    fn read(&self, file: &OpenFile, buf: &mut [u8]) -> Result<usize>;
    // 当用户调用write(fd,...)时
    fn write(&self, file: &OpenFile, buf: &[u8]) -> Result<usize>;
    // 其他如ioctl, lseek等
}
```
VFS层会维护一个`dev_file_ops`数组或哈希表，将**主设备号**映射到一个实现了`FileOps`的驱动对象上。

### 3.2 字符设备Core (Character Device Core)

**职责**:
-   作为所有字符设备驱动的注册中心。
-   将来自VFS层的请求，根据主设备号，路由到正确的驱动程序。

**设计与接口**:
```rust
// in char_device/mod.rs

// 所有字符设备驱动都必须实现这个trait
pub trait CharDeviceDriver: Send + Sync {
    // 实现VFS的FileOps
    fn read(&self, minor: u32, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, minor: u32, buf: &[u8]) -> Result<usize>;
    // ...
}

// CharDevCore是内核中的一个全局单例
pub struct CharDevCore {
    drivers: Spinlock<BTreeMap<u32, Arc<dyn CharDeviceDriver>>>, // major -> driver
}

impl CharDevCore {
    // 驱动注册接口
    pub fn register_driver(&self, major: u32, driver: Arc<dyn CharDeviceDriver>) -> Result<()>;

    // VFS调用的路由函数
    pub fn read(&self, major: u32, minor: u32, buf: &mut [u8]) -> Result<usize> {
        let drivers = self.drivers.lock();
        let driver = drivers.get(&major).ok_or(Error::NotFound)?;
        driver.read(minor, buf)
    }
    // write同理
}
```

### 3.3 TTY Core (终端业务逻辑层)

**职责**:
-   实现与终端相关的、但与具体硬件无关的通用逻辑。
-   核心是**线路规程 (Line Discipline)**，负责处理输入字符的编辑、回显、以及决定何时唤醒上层等待的任务。

**设计与接口**:
```rust
// in tty/mod.rs

// TTY Core需要下层UART驱动提供的接口
pub trait TtyLowLevelDriver: Send + Sync {
    // 启动硬件发送一个字符（用于回显）
    fn putc(&self, char: u8);
    // 其他可能的控制，如设置波特率等
}

// TTY Core自身，管理一个TTY会话
pub struct Tty {
    // ... 内部状态，如行缓冲区 ...
    line_buffer: Spinlock<Vec<u8>>,
    // 等待读操作的任务队列
    wait_queue: WaitQueue,
    // 指向下层的具体驱动
    low_level_driver: Arc<dyn TtyLowLevelDriver>,
}

impl Tty {
    // 当下层驱动（UART）收到字符时，调用此函数将字符注入TTY Core
    pub fn receive_char(&self, char: u8) {
        // 1. 实现回显
        self.low_level_driver.putc(char);

        // 2. 实现行编辑逻辑 (退格等)
        // ...

        // 3. 存入行缓冲区
        let mut buffer = self.line_buffer.lock();
        buffer.push(char);

        // 4. 判断是否满足唤醒条件 (如收到回车符)
        if char == b'\n' {
            self.wait_queue.wakeup_all();
        }
    }

    // 当上层(CharDevCore)请求读取时，调用此函数
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let mut buffer = self.line_buffer.lock();
            if !buffer.is_empty() && buffer.contains(&b'\n') {
                // 复制整行数据到buf
                // ...
                return Ok(bytes_copied);
            }
            // 否则，将当前任务放入等待队列并睡眠
            self.wait_queue.sleep();
        }
    }
}
```

### 3.4 UART驱动 (UART Driver)

**职责**:
-   直接与虚拟串口硬件的MMIO寄存器交互。
-   实现硬件中断服务程序（ISR）。
-   实现TTY Core要求的`TtyLowLevelDriver`接口。

**设计与接口**:
```rust
// in drivers/uart/virt_uart.rs

// 虚拟硬件寄存器布局
#[repr(C)]
struct UartRegs {
    rx_data: ReadOnly<u8>,
    tx_data: WriteOnly<u8>,
    status: ReadOnly<u8>,
    // ...
}

pub struct UartDriver {
    regs: *mut UartRegs,
    // 每个UART实例都关联一个TTY会话
    tty: Arc<Tty>,
}

impl UartDriver {
    // 中断服务程序
    pub fn handle_irq(&self) {
        // 1. 从状态寄存器确认是接收中断
        // 2. 从rx_data寄存器读取字符
        let char = unsafe { self.regs.as_ref().unwrap().rx_data.read() };
        // 3. 将字符注入到关联的TTY Core
        self.tty.receive_char(char);
    }
}

// 实现TTY Core要求的底层接口
impl TtyLowLevelDriver for UartDriver {
    fn putc(&self, char: u8) {
        // 1. 等待tx_data寄存器为空
        // 2. 将字符写入tx_data寄存器
        unsafe { self.regs.as_mut().unwrap().tx_data.write(char) };
    }
}
```

## 4. 完整调用链串联 (以`read`为例)

1.  **用户任务**: `read(fd, buf, len)` -> 触发系统调用。
2.  **VFS层**:
    *   通过`fd`找到对应的`OpenFile`对象。
    *   从`OpenFile`中得知主设备号`major`。
    *   调用`char_dev_core.read(major, minor, buf)`。
3.  **字符设备Core**:
    *   在`drivers`表中查找`major`对应的驱动。在这里，它会找到一个代表**TTY层**的驱动适配器。
    *   调用`tty_driver_adapter.read(minor, buf)`。
4.  **TTY Core**:
    *   这个适配器会找到`minor`号对应的`Tty`实例。
    *   调用`tty.read(buf)`。
    *   如果`tty`的行缓冲区没有完整的一行，当前任务就会在`tty.wait_queue`上**睡眠**。
5.  **中断发生 (用户按下按键)**:
    *   虚拟串口硬件触发中断。
    *   CPU跳转到中断处理入口，最终调用到`uart_driver.handle_irq()`。
6.  **UART驱动**:
    *   `handle_irq`从MMIO寄存器读取字符。
    *   调用`tty.receive_char(char)`将字符注入TTY层。
7.  **TTY Core (响应中断)**:
    *   `receive_char`处理字符（回显、编辑、缓存）。
    *   如果收到回车，它会调用`tty.wait_queue.wakeup_all()`**唤醒**之前睡眠的任务。
8.  **调用链返回**:
    *   被唤醒的任务从`tty.read()`中醒来，再次检查缓冲区，发现有数据了。
    *   它从行缓冲区拷贝数据到`buf`，然后逐层返回，最终`read`系统调用完成，用户任务拿到数据。

## 5. 实现时需要关注的要点

1.  **并发与锁**:
    *   所有可能被多核或中断上下文访问的共享数据（如`CharDevCore`的驱动列表、`Tty`的行缓冲区）都必须使用锁（如`Spinlock`）来保护。
    *   要特别注意锁的粒度和顺序，避免死锁。例如，在中断处理程序中获取锁要非常小心，避免长时间持有。
2.  **任务调度与同步**:
    *   必须实现一个健壮的等待队列（`WaitQueue`）机制，用于在没有数据时让任务睡眠，并在数据到达时准确地唤醒它们。
    *   这是实现阻塞式I/O的关键。
3.  **中断处理**:
    *   中断服务程序（ISR）必须尽可能快地执行。它的主要工作应该是从硬件取走数据，然后将耗时的处理（如复杂的行编辑）推迟到下半部（Tasklet/Workqueue）或直接在注入的函数中完成。
4.  **内存管理**:
    *   所有与硬件交互的缓冲区（如DMA缓冲区，虽然这里是虚拟MMIO）可能需要物理地址连续。
    *   内核与用户空间的数据拷贝必须通过安全的接口完成，防止非法内存访问。
5.  **抽象层的一致性**:
    *   确保所有接口（Traits）的设计足够通用，能够适应未来可能加入的新设备类型，而不仅仅是为当前的虚拟串口“量身定做”。

这份设计文档提供了一个清晰、分层且可扩展的框架。遵循这个设计，你可以一步步地为你的操作系统构建起一个健壮的字符设备I/O子系统。
