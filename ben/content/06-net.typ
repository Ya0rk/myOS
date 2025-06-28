#import "../components/prelude.typ": *
= 网络模块
<网络模块>

在网络模块中，我们使用了官方推荐的smoltcp库作为网络协议栈的底层处理，该库具有十分高效的网络协议栈处理，同时通过 Rust 内存安全、事件驱动模型和高度模块化设计，成为资源受限内核场景的理想网络栈。

为了体现linux中"一切皆文件"的思想，我们为网络系统中的关键结构体Socket实现了Filetrait，统一了网络和文件系统，方便内核设计中对其进行高效管理。在此基础上，我们对网络模块实现分层架构设计，从下至上依次为物理网络驱动层、传输层、Socket接口层。

目前在初赛阶段我们实现了一个简单的本地回环网络，但是由于时间原因，还未适配通过netperf、iperf网络测例，不过这也是我们在决赛阶段的一个目标，努力实现一个简洁高效、易于学习的网络系统模块。

== Socket套接字
<Socket套接字>

系统核心围绕SockMeta元数据结构和Socket trait展开，通过实现FileTrait将网络套接字无缝集成到文件系统中，实现了Linux"一切皆文件"的设计哲学。整个架构体现了协议无关性设计，TCP与UDP套接字通过统一的接口向上提供服务，底层差异由各自的具体实现处理。为了利用sockfd操控套接字，我们将socket映射到fdtable中：
#code-figure(
```rust
/// 将一个socket加入到fd表中
pub fn sock_map_fd(socket: Arc<dyn FileTrait>, cloexec_enable: bool) -> SysResult<usize> {
    let mut flag = OpenFlags::O_RDWR;
    let fdInfo = FdInfo::new(socket, flag);
    let new_info = fdInfo.off_Ocloexec(!cloexec_enable);
    let task = current_task().expect("no current task");
    let fd = task.alloc_fd(new_info)?;
    Ok(fd)
}
```,
    caption: [将 socket 映射到 fd 的函数],
    label-name: "sock-map-fd-fn",
)
SockMeta结构体作为套接字的核心元数据容器，采用Rust的强类型系统精确描述套接字状态。其字段设计反映了网络连接的全生命周期：从初始创建时的空端口和端点，到绑定后的本地地址确定，再到连接建立后的远程端点记录。特别是通过Option类型明确区分已初始化和未初始化状态，避免了传统C实现中常见的无效值问题。

#code-figure(
```rust
pub struct SockMeta {
    pub domain: Sock,
    pub iptype: IpType,
    pub recv_buf_size: usize,
    pub send_buf_size: usize,
    pub port: Option<u16>,
    pub shuthow: Option<ShutHow>,
    pub local_end: Option<IpEndpoint>,
    pub remote_end: Option<IpEndpoint>,
}
```,
    caption: [SockMeta 结构体],
    label-name: "sockmeta-struct",
)

Socket trait定义了完整的套接字操作接口，既包含标准的bind/connect/listen等基本操作，也提供了send_msg/recv_msg等增强功能。该trait标记为async_trait以适应现代异步IO需求，同时继承FileTrait实现了文件描述符的统一管理，可以通过Filetrait中的pollin、pollout异步检查网络缓冲区是否可读可写。

#code-figure(
```rust
#[async_trait]
pub trait Socket: FileTrait {
    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)>;
    async fn connect(&self, addr: &SockAddr) -> SysResult<()>;
    async fn send_msg(&self, buf: &[u8], dest_addr: &SockAddr) -> SysResult<usize>;
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)>;
    fn bind(&self, addr: &SockAddr) -> SysResult<()>;
    fn listen(&self, backlog: usize) -> SysResult<()>;
    fn set_recv_buf_size(&self, size: u32) -> SysResult<()>;
    fn set_send_buf_size(&self, size: u32) -> SysResult<()>;
    fn get_recv_buf_size(&self) -> SysResult<usize>;
    fn get_send_buf_size(&self) -> SysResult<usize>;
    fn shutdown(&self, how: ShutHow) -> SysResult<()>;
    fn get_sockname(&self) -> SysResult<SockAddr>;
    fn get_peername(&self) -> SysResult<SockAddr>;
    fn set_keep_alive(&self, action: u32) -> SysResult<()>;
    fn enable_nagle(&self, action: u32) -> SysResult<()>;
    fn get_socktype(&self) -> SysResult<Sock>;
}
```,
    caption: [Socket trait 定义],
    label-name: "socket-trait",
)

== Ethernet设备
<Ethernet设备>

Del0n1x使用NetDev结构体实现网络设备的封装和抽象，其包含两个关键组件：device字段表示具体的网络设备类型，当前支持环回接口；iface字段维护了访问smoltcp协议栈的通道。这种分离设计使得设备驱动与协议栈保持松耦合，未来扩展新设备类型时无需修改上层协议逻辑。

#code-figure(
```rust
pub enum NetDevType {
    Loopback(Loopback),
    Unspec,
}

pub struct NetDev {
    pub device: NetDevType,
    pub iface: Interface,
}
```,
    caption: [NetDevType 与 NetDev 结构体],
    label-name: "netdev-struct",
)

\u{25C6} 网络轮循

poll()方法是整个网络栈的驱动引擎，实现了事件处理的核心循环，每次poll轮循将处理SOCKET_SET中所有的句柄。

iface.poll()调用实现了三层重要功能：

- 接收处理：从设备读取数据包并递交给相应协议处理程序

- 发送处理：将协议栈待发送数据提交给设备驱动

- 状态更新：维护TCP定时器、重传队列等状态机

该方法主要在发送和就收数据的循环中使用，驱动协议栈与设备的异步交互。
#code-figure(
```rust
pub fn poll(&mut self) {
        let instant = Instant::from_millis(get_time_ms() as i64);
        let mut socket = SOCKET_SET.lock();
        let device = match self.device {
            NetDevType::Loopback(ref mut dev) => dev,
            NetDevType::Unspec => panic!("Device not initialized"),
        };
        self.iface.poll(instant, device, &mut socket);
    }
```,
    caption: [网络轮询 poll 方法],
    label-name: "net-poll-method",
)

== 传输层——UDP与TCP
<传输层udp与tcp>

TCP 是一种面向连接的字节流套接字，而UDP 是一种无连接的报文套接字。他们都继承了SockMeta中的字段，实现了不同的 Socket trait。

#code-figure(
```rs
pub struct TcpSocket {
    pub handle: SocketHandle,
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub state: SpinNoIrqLock<TcpState>,
}

pub struct UdpSocket {
    pub handle: SocketHandle,
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
}
```,
    caption: [TcpSocket 与 UdpSocket 结构体],
    label-name: "tcp-udp-socket-struct",
)

在Tcp、Udp数据结构中，都是用SocketHandle表示自己的句柄。该句柄存放在smoltcp提供的全局SOCKET_SET中，该set可以类比为fdtable，方便快速获取到该套接字的相关句柄，并通过该句柄实现与smoltcp底层网络栈功能交互。

#code-figure(
```rs
/// 全局handle管理器
pub static ref SOCKET_SET: SpinNoIrqLock<SocketSet<'static>> =
        SpinNoIrqLock::new(SocketSet::new(vec![]));

/// 通过闭包快速获取到对应的handle
pub fn with_socket<F, R>(&self, f: F) -> R
where
    F: FnOnce(&mut udp::Socket<'_>) -> R,
{
    let mut binding = SOCKET_SET.lock();
    let socket = binding.get_mut::<udp::Socket>(self.handle);
    f(socket)
}
```,
    caption: [SOCKET_SET 及 with_socket 函数],
    label-name: "socket-set-and-with-socket",
)

== Port端口分配
<Port端口分配>

在网络交互过程中，PortManager负责动态管理TCP/UDP端口资源的分配与回收。该系统采用多层级管理策略，在保证线程安全的前提下实现高效端口分配。为了记录端口分配清空，Del0n1x采用了双位图的设计，单个协议仅需8KB内存（65536位）即可实现O(1)复杂度的状态查询，相比传统哈希表节省了90%以上的内存开销。双位图设计彻底避免了TCP/UDP端口冲突。

#code-figure(
```rust
pub struct PortManager {
    /// 动态端口范围
    pub start: u16,
    pub end: u16,
    /// 回收端口队列
    pub recycled: VecDeque<u16>,
    /// TCP端口位图
    pub tcp_used_ports: BitVec,
    /// UDP端口位图
    pub udp_used_ports: BitVec,
}
```,
    caption: [PortManager 结构体],
    label-name: "port-manager-struct",
)

\u{25C6} 分配算法设计

alloc方法实现了三级分配策略，如算法所示。先从回收队列获取，，用局部性原理提升缓存命中率；然后随机尝试，PORT_RANGE范围内进行有限次随机探测；如果都失败，最后使用顺序扫描，保在极端情况下仍能穷尽搜索。

#algorithm-figure(
  pseudocode(
    no-number,
    [*function* alloc(domain: Sock) $\to$ Result<u16>],
    [*if* recycled queue is not empty *then*],
    ind,
    [port $<-$ recycled.pop_front()],
    [mark_used(domain, port)],
    [*return* port],
    ded,
    [chance $<-$ (end - start) - |recycled|],
    [*for* i = 0 *to* chance-1 *do*],
    ind,
    [random_port $<-$ start + (random() % PORT_RANGE)],
    [*if* try_mark_used(domain, random_port) *then*],
    ind,
    [*return* random_port],
    ded,
    ded,
    [*for* port $<-$ start *to* end *do*],
    ind,
    [*if* try_mark_used(domain, port) *then*],
    ind,
    [*return* port],
    ded,
    ded,
    [*return* error EADDRINUSE],
  ),
  caption: [端口分配三级策略],
  label-name: "port-alloc-strategy",
  supplement: [算法]
)