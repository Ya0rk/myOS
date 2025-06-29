#import "../components/prelude.typ": *

= 文件系统

== 虚拟文件系统

虚拟文件系统（Virtual File System，简称 VFS）是对各种文件系统的抽象，这种抽象屏蔽了各种具体文件系统的细节，为内核提供统一的统一的文件系统接口。在标准的 系统调用中，例如`open()`、`read()`、`write()`中，可以忽略掉文件系统的特性，调用统一的 VFS 接口，从而简洁、安全地实现对文件的各种操作。

我们充分利用 rust 语言的特性，借鉴 Linux 文件系统设计，以面向 trait（中文翻译为属性）的方式进行编程，提供了方便、安全的文件系统接口供内核的其他模块与用户系统调用使用。

Del0n1x OS 的虚拟文件系统的主要数据结构为`SuperBlock`、`Inode`、`Dentry`、`File`。

#figure(
  image("assets/vfs.png"),
  caption: [虚拟文件系统],
  supplement: [图],
)<文件系统>

=== SuperBlock

// ：
SuperBlock trait 超级块属性是一个具体的文件系统的抽象。每一个具体文件系统都有对SuperBlock trait 的实现，不同文件系统对同一个方法会有不同的实现。

超级块`SuperBlock trait`的定义如下：

#code-figure(
```rust
pub trait SuperBlockTrait: Send + Sync {
    /// 获取根节点
    fn root_inode(&self) -> Arc<dyn InodeTrait>;
    /// 将数据写回
    fn sync(&self);
    /// 显示文件系统信息
    fn fs_stat(&self) -> StatFs;
}
```,
    caption: [文件系统超级块],
    label-name: "Super block",
)

与C 语言相比，rust 提供了足够的抽象机制。对于不同的文件系统，实现 SuperBlock trait后就可以在文件系统中安全和高效地使用，鲜明的方法名称也为编程带来方便。 rust 同时区别于传统的面向对象语言，抛弃了继承机制的设计，鼓励面向 trait（翻译为属性） 编程，并使用组合替代继承，使得代码结构更为简单高效。

我们的内核中简化了超级块的提供的功能，仅提供有限的信息


=== Inode

索引节点（inode）是文件系统的核心，是文件的抽象。

索引节点属性 InodeTrait 是对索引节点（inode）的抽象，不同的文件系统有其自身的实现，但是其暴露出的方法是统一的。Linux 当中inode 结构体有 struct inode_operations \*i_op 字段，InodeTrait 与索引节点（inode）的 inode_operations功能类似，定义了索引节点（inode）应当实现的功能。

不同文件系统的 inode 结构体应当实现 InodeTrait。在 Linux 中，当处理内核文件系统操作或文件系统相关系统调用的时候，会获得对应的 inode 对象，随后去检查并调用 inode->i_op 中的回调函数来实现具体功能。这种面向方法（也可以成为面向对象）的编程思想极大地提高了内核编码的安全性和便利性，我们无需关心各个文件系统当中如何实现具体的方法的，我们仅仅需调用暴露的方法就好了。

rust 可以很方便地做到这一点，因为 rust 原生面向Trait 编程。我们充分利用了这一点！

我们定义了索引节点（Inode） 应当实现的方法 InodeTrait，具体的文件系统需要实现其中的方法，或者使用默认方法，就可以供上层使用

索引节点属性 `InodeTrait` 的主要内容如下：

#code-figure(
```rust
pub trait InodeTrait: Send + Sync {
    /// inode 的信息
    fn fstat(&self) -> Kstat;
    /// 在文件夹上创建一个子文件
    fn do_create(&self, bare_dentry: Arc<Dentry>, _ty: InodeType) -> Option<Arc<dyn InodeTrait>>;
    /// 读文件
    fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize;
    /// 写文件
    fn write_at(&self, _off: usize, _buf: &[u8]) -> usize;
    /// 截断文件
    fn truncate(&self, _size: usize) -> usize;
    /// unlink文件
    fn unlink(&self, valid_dentry: Arc<Dentry>) -> SysResult<usize>;
    /// link 文件
    fn link(&self, bare_dentry: Arc<Dentry>) -> SysResult<usize>;
    /// 获得page cache
    fn get_page_cache(&self) -> Option<Arc<PageCache>>;
    /// 更名
    fn rename(&self, old_path: Arc<Dentry>, new_path: Arc<Dentry>) -> SysResult<usize>;
    /// 文件夹读取目录项
    fn read_dents(&self) -> Option<Vec<Dirent>>;
    /// io 操作
    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize>
}
```,
    caption: [InodeTrait 接口定义],
    label-name: "inode-trait",
)

在文件对象 File 或者目录项对象中会持有数据类型为 `Arc<dyn InodeTrait>` 的索引节点（inode）对象。通过索引节点（inode）对象，,通过其实现的 Inode trait 接口我们可以获得文件的各种信息，进行文件的读写、创建、删除等操作。在调用 InodeTrait 定义的方法的时候，我们不需要关心其具体的数据类型，这一行为会自动分派给对应的文件系统的索引节点（inode）实现。


=== Dentry

目录项 Dentry 是目录树上节点的抽象。每一个有效的目录项持有对一个合法的索引节点（inode） 的引用操作系统应用程序使用路径获得对应的文件，这一过程是由目录项（dentry） 完成。目录项构成了一颗目录树，一般而言，通过从挂载节点向下搜索，实现对文件的查找。同时 Dentry 还对文件系统目录树进行了管理。

#figure(
  image("assets/访问目录树.png"),
  caption: [访问目录树],
  supplement: [图],
)<文件系统>

在初始化目录树的过程中，我们需要将具体的文件系统挂载（mount）在 目录树上，目录树提供了这一功能的实现。我们可以把任意的文件系统挂载到目录树上的文件夹节点上，将该目录项（dentry） 持有的 inode `替换`为该文件系统的根目录，我们就可以通过对应的路径，去访问该文件系统下的文件。

#figure(
  image("assets/mount_umount.png"),
  caption: [mount and umount],
  supplement: [图],
)<文件系统>

在我们的实现中，当我们挂载了一个新的文件系统到某个目录项上时，原有的目录项上持有的索引节点（inode）的引用会隐藏起来，当从这个目录项上卸载（unmount）该文件系统的时候原来持有的索引节点（inode）会恢复，进而恢复原有的子树结构。

目前仅支持内核中调用，未来我们希望实现 loop 设备后在用户态也可以使用

目录项`Dentry`的定义如下：
#code-figure(
    ```rust
    pub struct Dentry {
        /// 目录项文件名
        name: RwLock<String>,
        /// 对父dentry的弱引用
        parent: Weak<Dentry>,
        /// 孩子dentry的强引用
        children: RwLock<HashMap<String, Arc<Dentry>>>,
        /// 当前的持有的的inode对象
        inode: RwLock<Vec<Arc<dyn InodeTrait>>>,
        /// dentry的状态
        status: RwLock<DentryStatus>,
    }
    ```,
    caption: [Dentry结构],
    label-name: "dentry结构", 
)


通过children 字段获得当前目录项的子目录项，通过 parent 获得当前目录项的双亲目录项，注意到这里使用弱引用（不增加引用计数）防止出现循环引用。inode 字段为所持有的索引节点（inode）。

定义目录项状态DentryStatus
#code-figure(
```rust
pub enum DentryStatus {
    /// 这个 dentry 是有效的，并且已经初始化
    Valid,
    /// 这个 dentry 是有效的，但是没有初始化
    Unint,
    /// 这个 dentry 是无效的
    Negtive,
}
```,
    caption: [DentryStatus 枚举],
    label-name: "dentry-status",
)
目录项状态在这三个状态之间转移，当目录项被标记为无效时，会在合适的时机进行回收，并且释放对索引节点（inode）对象的引用。当目录项尚未初始化的时候，会在访问时初始化。只有当目录项有效时才可以进行访问。

目录项 （dentry）额外使用了一个缓存（Cache）用于加速从路径到目录项的查找，目录项缓存（DentryCache）使用内核定义的 Cache 泛型容器进行定义。目录项缓存的存在极大地加速了获得索引节点（inode）的过程，获得显著的性能提升。

#figure(
  image("assets/dentry_cache性能对比图.png"),
  caption: [dentry cache性能对比],
  supplement: [图],
)<文件系统>

=== File



文件对象（file）是进程中已打开文件在内粗那种的表示。文件由系统调用`open()`创建，由系统调用`close()`关闭。一个最基本的文件对象（file）由持有的索引节点（inode）、文件的偏移和其他状态信息组成。用户态程序调用 open 系统调用去创建文件对象时，首先需要根据路径获得对应的目录项（dentry），通过目录项获得索引节点（inode），将索引节点包装在文件对象（file）中，并设置其状态，最后将创建好的文件对象注册到文件描述符表。



== 磁盘文件系统

磁盘文件系统（Disk File System）是是操作系统中用于管理磁盘和其他存储设备上数据存储和组织的机制。其定义了数据如何以文件和目录的形式存储、命名、访问、更新，以及如何在物理存储介质上分配空间。

=== EXT4 文件系统

Ext4（第四代扩展文件系统）是Ext3文件系统的继承者，主要用于Linux操作系统。与前代文件系统相比，Ext4在性能、可靠性和容量方面都有显著改进。Ext4 文件系统对 Unix 操作系统适配性更好，支持硬链接等操作。

我们的操作系统使用了开源的 `lwext4-rust`库，为其提供可用的持久化设备，并利用库其提供的功能，根据 VFS 的接口设计进行适配与抽象。

== 非磁盘文件系统

非磁盘文件系统（Non-Disk File System）是指不依赖于传统磁盘存储介质的文件系统。在我们的操作系统中提供了若干个非磁盘文件系统供系统使用。

===  procfs

procfs 是一种特殊的文件系统，它不是从磁盘上的文件系统中读取数据，而是从内核中
读取数据。procfs 包括：

- `/proc/mounts` : 显示当前挂载的文件系统
- `/proc/meminfo` :  提供关于系统内存使用情况的信息，包括总内存、可用内存、缓存和缓冲区等详细数据
- `/proc/exe` : 当前正在运行的程序
- `/proc/self` : 当前正在运行的进程所持有的内容

这个文件系统完整地实现了 VFS 中所有的接口，用户可以`透明`地使用其中的文件

用户态程序可以很方便地从这些文件中提取相关信息。

=== devfs

devfs 中的文件代表一些具体的设备，比如终端、硬盘等。devfs 内包含：
- `/dev/zero` : 一个无限长的全 0 文件
- `/dev/null` : 用于丢弃所有写入的数据，并且读取时会立即返回 EOF（文件结束）
- `/dev/random` : 一个伪随机数生成器，提供随机数据流
- `/dev/rtc` : 实时时钟设备，提供日期和时间
- `/dev/tty` : 终端设备，能支持 ioctl 中的特定命令
- `/dev/loop0` : 回环设备，用于虚拟块设备


== 页缓存

页缓存（Page Cache）以页为单位缓存文件的内容。当我们需要读文件的时候，在缓存命中的情况下，就不需要去访问持久化设备，从而提高性能。当写文件的时候，也可以直接往页缓存中写，同时标记为脏页，就可以直接返回，而不需要等待数据真正地被写入到磁盘。脏页由内核进行统一的管理。总而言之，页缓存的设计极大地提高了文件的读写性能。

页缓存同时也是连接文件系统模块和内存模块的桥梁。用户可以调用 mmap 系统调用，将文件的页缓存映射到用户态地址空间中，用户就可以安全且高效地实现对文件的访问。 mmap设计 可以复用页缓存机制而安全高效地实现

在用户态程序借助文件进行进程间通信（IPC）这一常见场景下，性能也会获得极大地提升。

以下为页缓存（Page Cache）的定义

#code-figure(
```rust
pub struct PageCache {
    pub pages: RwLock<BTreeMap<usize, Arc<Page>>>,
    inode: RwLock<Option<Weak<dyn InodeTrait>>>,
}
```,
    caption: [PageCache 结构体],
    label-name: "page-cache",
)

在 PageCache 实现中以页对齐的地址为 key 去获得对应的页帧（Page）。

Ext4 文件系统的索引节点（inode）会持有这一页缓存，当不做特别要求的时候，均在页缓存中读写。系统会对页缓存进行管理，实现对齐其创建，映射与释放。并且当索引节点（inode）被释放的时候，其持有的页缓存（PageCache）也会自动释放，进而释放所持有的页帧。

== 其他数据结构

=== FdTable

用户态程序使用`open`系统调用打开文件后，会获得文件描述符（File Descriptor）来用于控制文件（File），这需要内核为每一个进程创建一个对应的文件描述符表（Fd Table）用于实现文件描述符转换到文件的映射

// #img(
//   image("../assets/todo", width: 70%),
//   caption: "一次从 read 系统调用的过程"
// )<phoenix-design>

以下为文件描述符表的实现：


#code-figure(
    ```rust
    pub struct FdTable {
    pub table: Vec<FdInfo>,
    pub rlimit: RLimit64,
    free_bitmap: Vec<u64>,
    next_free: usize,
    freed_stack: Vec<usize>,
    }

    #[derive(Clone)]
    pub struct FdInfo {
        pub file: Option<Arc<dyn FileTrait>>,
        pub flags: OpenFlags,
    }
    ```,
    caption: [FdTable结构],
    label-name: "FdTable",
)




#pagebreak()  // 强制分页