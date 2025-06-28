= 网络模块
<网络模块>

Phoenix操作系统的网络模块参考了Arceos的实现，但是Arceos的网络模块并不支持异步特性与IPV6协议，因此我们对其进行了较大改造，使其很好地兼容异步无栈协程架构的操作系统，并且顺利通过了libctest、netperf与iperf测试。

Phoenix使用了smoltcp库作为网络模块的基石，这是一个面向嵌入式设备的TCP/IP协议栈库。smoltcp以Rust语言编写，旨在提供高效、可扩展且易于集成的网络堆栈，非常适合资源受限的环境，不依赖于标准库，因此可以在无操作系统的`no_std`中运行。

== 数据链路层——Device设备
<数据链路层device设备>

Phoenix支持本地回环网络设备`Loopback`与`VirtIoNet`设备。所有的网络设备都需要实现`NetDevice`
trait
，这个trait为网络设备提供一个统一的接口，使得不同类型的网络设备可以通过相同的接口进行操作和管理。

```rust
pub trait NetDevice: Sync + Send {
    /// The capabilities of net device
    fn capabilities(&self) -> DeviceCapabilities;
    /// The ethernet address of the NIC.
    fn mac_address(&self) -> EthernetAddress;
    /// Whether can transmit packets.
    fn can_transmit(&self) -> bool;
    /// Whether can receive packets.
    fn can_receive(&self) -> bool;
    /// Size of the receive queue.
    fn rx_queue_size(&self) -> usize;
    /// Size of the transmit queue.
    fn tx_queue_size(&self) -> usize;
    /// Gives back the `rx_buf` to the receive queue for later receiving.
    fn recycle_rx_buffer(&mut self, rx_buf: Box<dyn NetBufPtrOps>)
        -> DevResult;
    /// Poll the transmit queue and gives back the buffers for previous
    /// transmiting.
    fn recycle_tx_buffers(&mut self) -> DevResult;
    /// Transmits a packet in the buffer to the network, without blocking,
    fn transmit(&mut self, tx_buf: Box<dyn NetBufPtrOps>) -> DevResult;
    /// Receives a packet from the network and store it in the [`NetBuf`],
    /// returns the buffer.
    fn receive(&mut self) -> DevResult<Box<dyn NetBufPtrOps>>;
    /// Allocate a memory buffer of a specified size for network transmission
    fn alloc_tx_buffer(
        &mut self,
        size: usize,
    ) -> DevResult<Box<dyn NetBufPtrOps>>;
}
```

为了使得网络设备符合`smoltcp`的接口需求，定义了`DeviceWrapper`结构体，它通过
`RefCell` 包装 `Box<dyn NetDevice>`，允许内部的可变访问，以便在实现
`Device` trait 时提供对底层设备的操作。

```rust
/// A wrapper for network devices, providing interior mutability for
/// `NetDevice`.
struct DeviceWrapper {
    /// The inner network device wrapped in a `RefCell` for interior
    /// mutability.
    inner: RefCell<Box<dyn NetDevice>>,
}
```

为了提供一个高层次的网络接口封装，定义了`InterfaceWrapper`结构体，包含设备和接口的详细信息，并实现相关的操作方法。它通过
`Mutex` 来保护对设备和接口的并发访问，确保线程安全。

```rust
/// A wrapper for network interfaces, containing device and interface details
/// and providing thread-safe access via `Mutex`.
struct InterfaceWrapper {
    /// The name of the network interface.
    name: &'static str,
    /// The Ethernet address of the network interface.
    ether_addr: EthernetAddress,
    /// The device wrapper protected by a `Mutex` to ensure thread-safe
    /// access.
    dev: Mutex<DeviceWrapper>,
    /// The network interface protected by a `Mutex` to ensure thread-safe
    /// access.
    iface: Mutex<Interface>,
}
```

== 网络层——IP
<网络层ip>

Phoenix支持Ipv4与Ipv6两种地址，在如`sys_bind`等系统调用中，操作系统需要接受用户传入的Ip地址，这里以Ipv4为例：

```rust
/// IPv4 address
pub struct SockAddrIn {
    /// always set to `AF_INET`
    pub family: u16,
    /// port in network byte order
    pub port: [u8; 2],
    /// address in network byte order
    pub addr: [u8; 4],
    pub zero: [u8; 8],
}
```

在POSIX规范中，系统调用传入的 Ipv4
address参数的端口为网络字节序，即大端序，而RISCV指令集为小端序，Phoenix在内核中使用`smoltcp`库提供的`IpEndpoint`结构体存储网络地址，以小端序存储。Phoenix为结构体实现了`From`
trait便于这些结构体进行转换。

```rust
pub struct IpEndpoint {
    pub addr: Address,
    pub port: u16,
}
```

Ip协议是网络层的核心协议，负责在不同网络之间传输数据包。数据包的接受、处理、发送以及路由处理的逻辑已经由`smoltcp`模块封装好了。

== 传输层——UDP与TCP
<传输层udp与tcp>

`smoltcp`库本身在tcp模块和udp模块就提供了相应的`Socket`的实现，为什么Phoenix内核重新封装了一遍`UdpSocket`和`TcpSocket`呢？

首先，这是为了提供更高层的抽象和接口，虽然 `smoltcp` 的 `udp::Socket` 和
`tcp::Socket`
提供了基本的UDP与TCP功能，但它的接口不完全符合Unix操作系统的需求。重新定义的
`UdpSocket`
与`TcpSocket`结构体，用`asycn`块封装实现了异步特性，并且提供了与POSIX标准类似的接口，并且这使得基于POSIX
API设计的应用程序更容易移植和使用。

此外，在内核的`UdpSocket`
与`TcpSocket`结构体中，使用`handle`字段表示套接字在全局 `SOCKET_SET`
中的句柄，便于在各种操作中快速访问和管理。通过存储
`SocketHandle`，可以将网络套接字的管理逻辑与具体的套接字操作分离。`SocketHandle`
提供了一个间接访问实际套接字对象的方式，使得 `UdpSocket`
结构体只需处理与套接字管理相关的逻辑，而不需关心具体的实现细节。

=== UDP
<udp>

内核中的Udp结构体除了存储`SocketHandle`外，还存储了本地地址`local_addr`与远程地址`peer_addr`，均使用`RwLock`
提供并发读写保护，`Option` 类型允许本地地址未绑定与远程地址未连接时为
`None`。`nonblock`用于指示套接字是否处于非阻塞模式，`AtomicBool`
提供原子操作，确保在多线程环境下对非阻塞模式标志进行安全的读写操作。

```rust
// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    /// Handle obtained after adding the newly created socket to SOCKET_SET.
    handle: SocketHandle,
    /// Local address and port. Uses RwLock for thread-safe read/write access.
    local_addr: RwLock<Option<IpListenEndpoint>>,
    /// Remote address and port. Uses RwLock for thread-safe read/write
    /// access.
    peer_addr: RwLock<Option<IpEndpoint>>,
    /// Indicates if the socket is in nonblocking mode. Uses AtomicBool for
    /// thread-safe access.
    nonblock: AtomicBool,
}
```

=== Tcp
<tcp>

`TcpSocket`结构体大体上与`UdpSocket`类似。

```rust
/// A TCP socket that provides POSIX-like APIs.
pub struct TcpSocket {
    /// Manages the state of the socket using an atomic u8 for lock-free
    /// management.
    state: AtomicU8,
    /// Indicates whether the read or write directions of the socket have
    /// been explicitly shut down.
    shutdown: UnsafeCell<u8>,
    /// An optional handle to the socket
    handle: UnsafeCell<Option<SocketHandle>>,
    /// Stores the local IP endpoint of the socket
    local_addr: UnsafeCell<IpEndpoint>,
    /// Stores the peer IP endpoint of the socket
    peer_addr: UnsafeCell<IpEndpoint>,
    /// Indicates whether the socket is in non-blocking mode, using an atomic
    /// boolean for thread-safe access.
    nonblock: AtomicBool,
}
```

这里主要提一下与往届作品不同的创新点：

+ #strong[`TcpSocket`借用了原子变量实现了无锁管理]
  往届作品中，基本上都是使用`Mutex`这种自旋锁对Tcp套接字进行并发安全处理，Phoenix使用`AtomicU8`原子变量表示套接字的状态
  ，允许多个线程在不使用锁的情况下安全地对套接字状态进行读取和修改。相比传统的锁机制（如
  `Mutex` 或
  `RwLock`），原子操作更轻量级，减少了上下文切换和线程阻塞，从而提高了并发性能。Tcp的状态更新操作时不可分割的，Phoenix使用`compare_exchange`
  方法可以在读取和更新状态之间保证操作的原子性，避免竞态条件。

  ```rust
fn update_state<F, T>(
    &self,
    expect: u8,
    new: u8,
    f: F,
) -> Result<SysResult<T>, u8>
where
    F: FnOnce() -> SysResult<T>,
{
    match self.state.compare_exchange(
        expect,
        STATE_BUSY,
        Ordering::Acquire,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            let res = f();
            if res.is_ok() {
                self.set_state(new);
            } else {
                self.set_state(expect);
            }
            Ok(res)
        }
        Err(old) => Err(old),
    }
}
  ```

+ #strong[使用`ListenTable`进行高效的端口管理]
  `ListenTable`使用一个大小为 65536 的数组来管理每个可能的 TCP
  端口，每个端口对应一个
  `ListenTableEntry`。这种设计通过数组的索引来直接访问监听条目，使得查找和操作非常高效。

  ```rust
/// A table for managing TCP listen ports.
/// Each index corresponds to a specific port number.
pub struct ListenTable {
    /// An array of Mutexes, each protecting an optional
    /// ListenTableEntry for a specific port.
    tcp: Box<[Mutex<Option<Box<ListenTableEntry>>>]>,
}
  ```

  `ListenTableEntry` 中的 `syn_queue` 用于管理在三次握手过程中收到的 SYN
  包，等待其连接建立完成。`waker`
  用于在有新连接到来时唤醒对应的监听套接字，从而处理新连接请求。这种设计确保了系统可以高效地管理和处理大量的并发连接请求。

  ```rust
/// An entry in the listen table, representing a specific listening
/// endpoint.
struct ListenTableEntry {
    /// The IP address and port being listened on.
    listen_endpoint: IpListenEndpoint,
    /// The SYN queue holding incoming TCP connection handles.
    syn_queue: VecDeque<SocketHandle>,
    /// The waker used to wake up the listening socket when a new
    /// connection arrives.
    waker: Waker,
}
  ```

+ #strong[支持Ipv4和Ipv6的灵活监听] 

  `ListenTableEntry` 的 `can_accept`
  方法支持 #strong[IPv4-mapped IPv6 addresses];，使得 在特殊情况下IPv6
  套接字可以接受 IPv4
  的连接。这种设计提供了更大的灵活性，允许在同一个端口上同时监听 IPv4 和
  IPv6 的连接。

== 套接字API
<套接字api>

为了统一UDP套接字、TCP
套接字和Unix套接字，Phoenix内核对其再次进行了封装在操作系统中。

```rust
pub enum Sock {
    Tcp(TcpSocket),
    Udp(UdpSocket),
    Unix(UnixSocket),
}

pub struct Socket {
    /// The type of socket (such as STREAM, DGRAM)
    pub types: SocketType,
    /// The core of a socket, which includes TCP, UDP, or Unix domain sockets
    pub sk: Sock,
    /// File metadata, including metadata information related to sockets
    pub meta: FileMeta,
}
```

文件和套接字通常都作为文件描述符处理，以统一的方式进行读写和管理。为
`Socket` 结构体实现 `File` trait
，套接字可以像普通文件一样进行读写操作，方便管理和使用。Phoenix的 `File`
trait为异步函数，通过实现异步读写方法，可以利用 Rust
的异步特性，提高套接字操作的效率。

Phoenix对网络模块中的资源管理充分使用了RAII思想，确保在创建和销毁时正确地分配和释放资源，当`Socket`被`drop`时，会通过`shutdown`关闭套接字并从全局的`SOCKET_SET`中移除套接字句柄，确保系统不再持有对该套接字的引用，防止资源泄漏。

Phoenix内核中的各种系统调用如`sys_send`、`sys_recv`、`sys_write`均通过`Socket`和`Sock`结构体封装的API进行实现。

与往届一等奖参赛作品如Alien、Titanix等相比，Phoenix#strong[提供了更加高效的对套接字的`Poll`操作];，以Tcp为例，Phoenix通过调用`smoltcp`的`poll_dealy`方法检查多个条件（如连接状态、是否有数据需要传输、ACK
状态、超时等）来决定下次调用 `poll`
的时间，得到时间后，新构建一个定时器`PollTimer`放到`TimerManager`中，当时钟中断到来时检查如果定时器超时了，调用回调函数`callback`自动进行`poll`操作

```rust
impl InterfaceWrapper {
    pub fn check_poll(&self, 
                      timestamp: SmolInstant, 
                      sockets: &Mutex<SocketSet>)
    {
        /* skip */
        match iface.poll_delay(timestamp, &mut sockets)
        {
            Some(Duration::ZERO) => iface.poll(/* skip */),
            Some(delay) => {
                let next_poll = /* caculate by `delay` */
                if next_poll < current {
                    iface.poll(/* skip */);
                } else {
                    let timer = 
                        Timer::new(next_poll, Box::new(PollTimer {}));
                    TIMER_MANAGER.add_timer(timer);
                }
            }
            None => {
                let next_poll = /* 2 milliseconds later */
                let timer = Timer::new(next_poll, Box::new(PollTimer {}));
                TIMER_MANAGER.add_timer(timer);
            }
        }
    }
}

struct PollTimer;

impl TimerEvent for PollTimer {
    fn callback(self: Box<Self>) -> Option<Timer> {
        poll_interfaces();
        None
    }
}
```
