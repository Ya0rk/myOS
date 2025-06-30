#import "../components/prelude.typ": *
= 内存管理

== 地址空间总览

=== 地址空间布局

Del0n1x OS 的地址空间大小为$2^39 B = 512 G B$，布局如图所示：

// 【图片（用户、RV、LA）】
#figure(
    image("assets/地址空间布局.png"),
    caption: [地址空间布局],
    supplement: [图]
)

#h(2em)Del0n1x 的用户程序与内核程序共用同一个地址空间。用户态程序只允许访问用户地址空间，即地址空间的低半部分。内核态程序不仅可以访问用户地址空间，还对内核地址空间，即地址空间高半部分具有访问权。

用户地址空间使用帧映射，即由内核分配对应的物理页帧加以映射，以保证高效的物理内存利用率。而内核地址空间中还存在对物理内存空间的直接映射（偏移映射），将物理地址`pa`映射为`pa + KERNEL_PG_ADDR_BASE`，便于内核直接访问和操作物理内存。RISC-V架构与LoongArch架构的物理内存布局有所不同，故两个架构下地址空间的布局也不尽相同。

=== 地址翻译模式

LoongArch 架构支持“分页地址翻译模式”和“直接映射地址翻译模式”两种地址翻译模式。其中，分页地址翻译模式即为依赖MMU遍历页表的地址翻译模式，而直接映射地址翻译模式则是将落在直接映射窗口中的虚拟地址直接转换为对应的物理地址。Del0n1x 设置了`0x8000_xxxx_xxxx_xxxx`和`0x9000_xxxx_xxxx_xxxx`两个内核态可用的直接映射窗口，分别用于设备和物理内存的直接访问，窗口内的虚拟地址将被映射到`0x0000_xxxx_xxxx_xxxx`。

RISC-V 架构只支持分页地址翻译模式。Del0n1x 在 RISC-V 和 LoongArch 两种架构下均使用 SV39 分页翻译模式，页大小为`4096B`。内核在初始化时，将设备、物理内存映射到内核地址空间的对应区域。

=== Boot阶段的预映射

RISC-V 架构的 QEMU virt 机器上，CPU 默认不开启分页地址翻译模式，而是直接访问物理地址。但是 Del0n1x 内核链接的基地址`0xffffffc080200000`位于 SV39 模式下的高半段地址空间，是 Boot 初期不可达的虚拟地址。为了解决这个冲突，Del0n1x 在 Boot 阶段中创建了一个临时页表，并为内核地址空间映射了巨页，这样就可以在执行内核代码之前开启MMU的分页地址翻译模式，保证了内核中地址的正确性和有效性。这个临时页表将在内核初始化阶段被淘汰。
#code-figure(
    ```riscv
_start:
    ...

    # satp: 8 << 60 | boot_pagetable （开启页表机制 SV39）
    la t0, boot_pagetable
    li t1, 8 << 60
    srli t0, t0, 12
    or t0, t0, t1
    csrw satp, t0
    sfence.vma

    ...

    boot_pagetable:
    # 这是大页表
    # 里面只需要两个pte，供我们找到正确的物理地址
    # 0x0000_0000_8000_0000 -> 0x0000_0000_8000_0000

    .quad 0
    .quad 0
    .quad (0x80000 << 10) | 0xcf # VRWXAD
    .zero 8 * 255
    .quad (0x80000 << 10) | 0xcf # VRWXAD
    .zero 8 * 253
```,
    caption: [RISC-V entry.S中的预映射],
    label-name: "riscv-premap"
)


#h(2em)LoongArch 架构下，Del0n1x 内核链接的基地址为`0x9000000000200000`，这是一个直接映射地址翻译模式下的虚拟地址。Del0n1x 只需在 Boot 阶段设置直接映射窗口，MMU 就能正确解析内核中所有地址。

#code-figure(
```loongarch
    ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
    lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
    csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
    ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
    lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
    csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1
```  ,
    caption: [LoongArch 直接映射窗口设置],
    label-name: "loongarch-premap"
)


=== 内核地址转换

Del0n1x在对 LoongArch 架构的兼容中同时使用了两种地址翻译模式，并且两种模式均建立了一个虚拟地址到物理地址的直接映射，这将导致存在两个虚拟地址映射到同一个物理地址。在使用两种地址翻译模式的边界上，需要对两种虚拟地址进行转换。同时，由于直接映射的存在，内核代码中需要大量地进行物理地址和虚拟地址之间的转换。为了避免在代码中插入过多的加减运算式，Del0n1x 充分发挥 Rust 语言的优势，使用如下 trait 实现内核中地址之间的转换：

#code-figure(
```rust
// os/src/mm/address.rs

/// 直接映射地址翻译模式下的地址
pub trait Direct {
    /// 翻译为物理地址
    fn direct_pa(&self) -> PhysAddr;
    /// 转换为分页地址翻译模式下的地址
    fn paged_va(&self) -> VirtAddr;
}

/// 分页地址翻译模式下的地址
pub trait Paged {
    /// 翻译为物理地址
    fn paged_pa(&self) -> PhysAddr;
    /// 转换为直接映射地址翻译模式下的虚拟地址
    fn direct_va(&self) -> VirtAddr;
}

// os/src/hal/[ARCH]/mem/address.rs
impl Direct for VirtAddr {...}
impl Direct for PhysAddr {...}
impl Paged for VirtAddr {...}
impl Paged for PhysAddr {...}
```    ,
    caption: [内核地址类型转换接口],
    label-name: "kernel-addr-trait"
)


#h(2em)藉此，只需要知道源地址和目标地址使用的地址翻译模式，就可以通过简单的调用方法来进行转换。在内核通过直接映射访问物理内存的场景下，这个设计统一了地址翻译和等价虚拟地址转换的接口，极大地优化了代码的可读性。

#figure(
    image("assets/直接映射地址转换.png"),
    
    caption: [内核地址转换示意],
    supplement: [图]
)

#h(2em)由于 RISC-V 架构下只有一种地址翻译模式，也就只有一个虚拟地址到物理地址的直接映射，因此令两个trait的实现完全相同即可。

== 物理内存管理

=== 物理页帧分配器

Del0n1x的内核管理了所有可分配的物理内存。在分页地址翻译模式中，当创建一个新的帧映射时，内核需要为虚拟内存页分配对应的物理页帧；在创建文件页缓存时，内核也需要为缓存的页分配物理页帧。目前的设计下，Del0n1x继承了rCore使用的StackFrameAllocator分配器：

#code-figure(
```rust
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}
/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}
impl FrameAllocator for StackFrameAllocator {...}
```    ,
    caption: [Stack Frame Allocator分配器],
    label-name: "stack-frame-allocator"
)

#h(2em)该分配器使用了一种最简单的物理页帧分配策略：记录从未被分配的物理页号区间和已回收的物理页号栈，栈不为空时从栈中分配物理页号对应的页帧，栈中页号用尽时则从未分配区间的起点处分配。当`dealloc`方法被调用时，待回收的物理页号将被压入已回收物理页号栈，等待下一次被分配。

=== 物理页帧的生命周期管理

Del0n1x继承了rCore中的`FrameTracker`类型，使用RAII的思想维护已分配的物理页帧。`FrameTracker`初始化时自动调用`FrameAllocator::alloc`方法分配物理页号，并将对应物理页帧清零；析构时自动调用`FrameAllocator::dealloc`方法，回收物理页帧。

#code-figure(
```rust
impl FrameTracker {
    pub fn new() -> Option<Self> {
        FRAME_ALLOCATOR.lock().alloc().map( | ppn | {
            let bytes_array = ppn.get_bytes_array();
            for i in bytes_array {
                *i = 0;
            }
            Self { ppn }
        })
    }
}
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self.ppn);
    }
}
```,

    caption: [物理页帧分配器的RAII管理],
    label-name: "frame-allocator-raii"        
)



== 进程内存管理

=== 虚拟内存区域

在Del0n1x中，进程内存管理的基础单元是虚拟内存区域（Virtual Memory Area，VMA）。Del0n1x将虚拟地址空间上映射的一段连续区域抽象为一个VMA，这一段区域有统一的映射类型、访问权限、共享标记等属性。在内核中，进程的程序映射区域、栈、堆、一个文件映射区域、一个共享内存区域等等，分别都可以抽象为一个VMA。Del0n1x 使用`VmArea`结构体管理 VMA ，并通过 RAII 机制管理相关资源的释放：

#code-figure(
```rust
pub struct VmArea {
    /// VMA 虚拟地址范围
    range_va: Range<VirtAddr>,
    /// 使用 btreemap 持有并关联物理页
    pub pages: BTreeMap<VirtPageNum, Arc<Page>>,
    /// VMA 访问权限
    pub map_perm: MapPerm,
    /// VMA 类型
    pub vma_type: VmAreaType,

    /// 文件映射：保存映射标志位
    pub mmap_flags: MmapFlags,
    /// 文件映射：持有映射的文件
    pub mmap_file: Option<Arc<dyn FileTrait>>,
    /// 文件映射：保存映射的文件偏移
    pub mmap_offset: usize,

    /// 是否共享
    pub shared: bool,
}
```    ,
    caption: [虚拟内存区域结构体],
    label-name: "virtual-memory-area-def"
)


=== 进程地址空间

Del0n1x 使用`MemorySpace`结构体管理每个进程的地址空间。该结构体定义如下：
#code-figure(
```rust
pub struct MemorySpace {
    /// 进程页表
    page_table: SyncUnsafeCell<PageTable>,
    /// 持有进程地址空间中的所有虚拟内存区域(VMA)，析构时释放这些VmArea结构体
    areas: SyncUnsafeCell<RangeMap<VirtAddr, VmArea>>,
}
```,
    caption: [进程地址空间结构体],
    label-name: "proc-memory-space"    
)

#h(2em)其中，`PageTable`结构体是分页地址翻译模式下进程页表的抽象。该结构体存储页表根目录的物理页号用于地址空间切换，并持有所有目录页的`FrameTracker`以便在析构时回收这些物理页帧：
#code-figure(
```rust
pub struct PageTable {
    pub root_ppn: PhysPageNum,
    pub frames: Vec<FrameTracker>,
}
```,
    caption: [Page Table结构体],
    label-name: "pagetable-struct"    
)


== 缺页异常处理

=== 进程缺页异常概述

Del0n1x 的设计中，缺页异常主要来源于以下四种情况：

#list(
    [访问了未映射或无权限的虚拟地址],
    [访问了未分配的区域],
    [尝试写入了写时复制（Copy on Write，CoW）的区域],
    [访问了未缓存的区域],
    indent: 4em
)
// - 
// - 
// - 
// - 

#figure(
    image("assets/缺页异常处理.png"),
    caption: [进程缺页异常总览],
    supplement: [图]
)

#h(2em)Del0n1x 的内存管理模块能够正确地对缺页异常进行处理，从而使用写时复制机制、懒分配机制、文件页缓存等提高整体效率。

=== 写时复制机制

当有多个进程可写访问同一内存资源（如文件缓存、物理内存页）但又不希望在进程间共享修改时，写时复制（Copy on Write，CoW）机制能推迟复制操作的发生，直到任意一方尝试进行修改操作。 CoW 机制能显著提升操作系统的时空效率。Del0n1x 依赖缺页异常处理实现了对 CoW 机制的支持，并全面使用 CoW 机制以保证进程`fork`、`mmap`文件私有映射等功能的高效。

Del0n1x 借用页表项（Page Table Entry，PTE）上未使用的标志位标识 CoW 的内存页。当内存页需要被标记为 CoW 时，内核将遍历页表，将对应 PTE 的 CoW 标志位设置为 1，同时摘除 PTE 上的可写标志位。当进程尝试修改该内存页的内容时，会因为不满足权限触发缺页异常，在异常处理函数中恢复 PTE 的标志位。Del0n1x 依赖 Rust 原子引用计数的强大功能，可以判断缺页异常触发时是否仍然存在内存页的共用，在存在共用时执行复制，在不存在共用时（如 fork 之后父子进程的其中一个已终止或执行了 exec）仅恢复可写标志位，优化了`fork + exec`的时间效率。

=== 懒分配机制

//
懒分配（Lazy Allocation）机制将进程堆栈、mmap匿名映射等区域分配内存页的时机延迟到访问时。Del0n1x 会为每个进程创建一个`8 MB`大小的用户栈，分配全部`8 MB`的内存页会带来不可忽视的时空开销。应用懒分配机制后，用户栈分配的时间开销分散在运行时，同时未被使用的栈空间将永远不会分配。同理，mmap匿名映射也应用懒分配机制降低时空开销。

懒分配机制的实现同样依赖于缺页异常处理。当进程访问到未分配内存页的虚拟地址时，会触发缺页异常，Del0n1x 的缺页处理函数将立即为该虚拟页分配对应的物理内存页并映射到页表。

== 内核动态内存分配

为了在Del0n1x 内核中使用Rust `alloc`库提供的`Vec`、`Arc`、`BTreeMap`等数据结构，避免重复造轮子，Del0n1x 继承了 rCore 中使用的伙伴分配器（Buddy Allocator）用于内核堆的动态分配。伙伴分配器能够针对待分配空间的布局（Layout），在内核堆里寻找连续的未分配区域进行分配。值得注意的是，如果一次性申请分配过大的动态内存空间，可能导致伙伴分配器找不到足够大的连续区域而出错。为此，Del0n1x 对申请内核堆空间的场景做了预判断，只分配必要的空间，防止此类问题的发生。

== 用户地址检查

为了响应用户请求、正确传达信息，内核往往需要频繁访问用户态传入的地址。Del0n1x 采用了用户程序与内核程序共用地址空间的设计，内核可以直接通过 MMU 解引用用户态虚拟地址进行操作。然而，传入的地址中可能存在非法地址。如果不做处理，轻则可能导致内核`panic`，重则可能导致内核被侵入，用户窃取最高权限。因此，检查传入地址的合法性是内核设计必须考虑的问题。

用户态传入地址不合法主要有以下两种情况：

#enum(
    enum.item(1)[传入了用户没有访问权限的地址，如内核`.data`段、内核堆],
    enum.item(2)[传入了未映射或无读写权限的地址],
    indent: 2em
)
// 1. 传入了用户没有访问权限的地址，如内核`.data`段、内核堆
// 2. 传入了未映射或无读写权限的地址

#h(2em)第一种情况可以使用简单的地址数值判断解决。针对第二种情况，Del0n1x 参考了往届队伍的做法，依赖内核 Trap 中的缺页异常处理逻辑解决。Del0n1x 为每一个 CPU 核心定义一个`ktrap_ret`变量（Kernel Trap Return-value），用于存储内核 Trap 的执行结果。
#code-figure(
```rust
pub struct CPU{

    ...
    /// None: 没有发生kernel trap
    /// Some(Ok): 发生kernel trap，正常处理（如CoW、懒分配、文件未缓存）
    /// Some(Err): 发生kernel trap，处理异常，说明存在非法访问
    /// 目前仅支持保存缺页异常的结果
    ktrap_ret: Option<SysResult<()>>,
    ...

}
```,
    caption: [ktrap_ret 定义],
    label-name: "ktrap_ret-def"
)

#h(2em)Del0n1x 向待检查地址读/写一个字节，并获取`ktrap_ret`的内容，据此判断地址所在的虚拟内存页是否合法。如果需要检查一段连续的地址范围是否合法，则将其拆分为数个内存页分别进行读/写检查即可。
#code-figure(
```rust
pub fn try_load_page(addr: VirtAddr) -> SysResult<()> {
    #[cfg(target_arch = "riscv64")]
    unsafe fn try_load_page_inner(addr: usize) {
        asm!(
            "mv t0, a0",
            "lb t0, 0(t0)",
            in("a0") addr,
            out("t0") _,
        );
    }

    #[cfg(target_arch = "loongarch64")]
    unsafe fn try_load_page_inner(addr: usize) {...}

    // get_current_cpu().ktrap_ret.take()
    take_ktrap_ret();
    unsafe {
        try_load_page_inner(addr.0);
    }
    // get_current_cpu().ktrap_ret.take().map_or(Ok(()), |ret| ret)
    take_ktrap_ret().map_or(Ok(()), |ret| ret)
}

```,
    caption: [用户地址检查],
    label-name: "user-addr-check"
 )


#pagebreak()