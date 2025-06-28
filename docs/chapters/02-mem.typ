#import "../template.typ": img

= 内存管理

== 物理内存管理

=== 内核动态内存分配器

Phoenix 使用伙伴分配器管理内核所需的动态内存结构，来自 crate
`buddy_system_allocator`。

伙伴分配器（Buddy Allocator）是一种内存分配算法，常用于操作系统内核和高性能应用程序中，通过分配和管理内存块来满足不同大小的内存请求，并进行高效的合并和分割操作。其工作原理是将内存块按照 2 的幂次方大小分为多个层级，当需要分配特定大小的内存时，从最小适合该请求的层级开始查找。每个内存块都有一个“伙伴”块，如果块大小是 $2^k$，那么它的伙伴块也是 $2^k$ 且紧挨着它。通过检查和计算伙伴块的地址，可以快速地进行内存分割与合并。当需要分配的内存块小于当前可用最小块时，将当前块一分为二，直到找到合适大小的块为止；当释放一个内存块时，若其伙伴块也空闲，则将两个块合并为一个更大的块，递归进行直到不能再合并为止。伙伴分配器的优点在于分配和释放内存块的操作非常快速，且通过内存块大小的选择和合并操作，有效减少了外部碎片。

=== 物理页分配器

内核还需要管理全部的空闲物理内存，Phoenix 为此使用了来自 rCore 的仓库的 `bitmap-allocator`。Phoenix 在内核初始化时，会将所有内核未占用的物理内存加入物理页分配器。

Bitmap allocator 的主要原理是通过一个位图来管理一段连续的内存空间。这个位图中的每一位代表一块内存，如果该位为 0，说明对应的内存块空闲；如果该位为 1，说明对应的内存块已经被分配出去。当需要分配一个指定大小的内存时，bitmap allocator 首先检查位图中是否有足够的连续空闲内存块可以满足分配请求。如果有，就将对应的位图标记为已分配，并返回该内存块的起始地址；如果没有，就返回空指针，表示分配失败。当需要释放已经分配出去的内存时，bitmap allocator 将对应位图标记为未分配。这样，已经释放的内存块就可以被下一次分配请求使用了。

此外，Phoenix 将物理页帧抽象成 `FrameTracker` 结构体，并结合 RAII 的思想，在结构体析构时自动调用 `dealloc_frame` 函数将页帧释放。

```rust
/// Manage a frame which has the same lifecycle as the tracker.
pub struct FrameTracker {
    /// PPN of the frame.
    pub ppn: PhysPageNum,
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc_frame(self.ppn);
    }
}
```

== 地址空间

=== 地址空间布局

Phoenix 地址空间的设计如下图所示：

#img(
    image("../assets/address-space.png"),
    caption: "地址空间"
)<address_layout>

Phonix 内核态页表保存在全局内核地址空间 `KERNEL_SPACE` 中，用户地址空间共享内核二级页表。

对于内核地址空间，为了方便管理所有物理地址，采用偏移映射的方式将物理地址以加上偏移量的方式映射为虚拟地址，即每一个虚拟地址都为对应物理地址加上`VIRT_RAM_OFFSET`。

对于用户地址空间，为了利用分页机制的灵活性，消除外部碎片，采用随机映射的方式将需要的页随机映射到空闲物理内存中随机一块页帧。

=== Boot 阶段高位映射

在 QEMU 平台上，内核的入口地址位于 0x8020_0000。在 xv6 与 rCore-Tutorial 中，0x8020_0000 以上部分以直接映射的方式作为内核地址空间，0x8000_0000 以下部分以随机映射的方式作为用户程序地址空间。因此用户程序最多只有 2G 的地址空间，而考虑到 MMIO 也映射在低位，用户程序地址空间只会更少。因此，Phoenix 充分利用页表的灵活性，将内核地址空间以偏移映射的方式映射到 RISC-V SV39 规定的高位地址空间，即 0xFFFF_FFC0_0000_0000 至 0xFFFF_FFFF_FFFF_FFFF，每一个虚拟地址对应于物理地址加上一个相同的偏移量，而将用户地址空间映射到 0x0 至 0x3F_FFFF_FFFF。这样，用户地址空间不足的问题就被解决了。

但是问题也随之而来，OpenSBI 会识别内核 ELF 文件入口地址的加载地址（LMA），然后在启动完毕后会跳转到此处，这时 RISC-V 页表机制还未开启，内核会直接访问物理地址。而开启页表时需要保证指令在启动页表前后物理地址连续，这就需要跳板页。Phoenix 希望在内核尽早映射到高位地址空间，这样在调试时能够尽早将代码与地址对应，因此 Phoenix 首先将内核虚拟地址链接到高位地址空间，将加载地址链接到低位，然后结合 RISC-V 巨页的机制，在内核启动阶段，在 Boot 页表中构造了三个巨页，地址分别为 0x8000_0000、0xFFFF_FFC0_0000_0000 和 0xFFFF_FFC0_8000_0000。其中，0x8000_0000 所在巨页作为跳板页，保证在打开页表前后 `pc` 寄存器指向连续的物理地址；0xFFFF_FFC0_0000_0000 所在巨页保存了 MMIO 的高位映射，以便在 Boot 页表阶段开启打印调试功能；0xFFFF_FFC0_8000_0000 所在巨页对应于内核 ELF 文件的虚拟地址高位映射，在 `_start` 函数最后会跳转到此处执行内核代码。在`entry.rs`代码执行完毕后，内核便成功执行在高位地址空间了，最后，Boot 页表会在 `main` 函数执行过程中被更加细化的全局内核页表 `KERNEL_SPACE` 所取代。

```rust
// arch/src/riscv64/entry.rs

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; KERNEL_STACK_SIZE * MAX_HARTS] =
    [0u8; KERNEL_STACK_SIZE * MAX_HARTS];

#[repr(C, align(4096))]
struct BootPageTable([u64; PTES_PER_PAGE]);

static mut BOOT_PAGE_TABLE: BootPageTable = {
    let mut arr: [u64; PTES_PER_PAGE] = [0; PTES_PER_PAGE];
    arr[2] = (0x80000 << 10) | 0xcf;
    arr[256] = (0x00000 << 10) | 0xcf;
    arr[258] = (0x80000 << 10) | 0xcf;
    BootPageTable(arr)
};

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(hart_id: usize, dtb_addr: usize) -> ! {
    core::arch::asm!(
        // 1. set boot stack
        // sp = boot_stack + (hartid + 1) * 64KB
        "
            addi    t0, a0, 1
            slli    t0, t0, 16              // t0 = (hart_id + 1) * 64KB
            la      sp, {boot_stack}
            add     sp, sp, t0              // set boot stack
        ",
        // 2. enable sv39 page table
        // satp = (8 << 60) | PPN(page_table)
        "
            la      t0, {page_table}
            srli    t0, t0, 12
            li      t1, 8 << 60
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
        ",
        // 3. jump to rust_main
        // add virtual address offset to sp and pc
        "
            li      t2, {virt_ram_offset}
            or      sp, sp, t2
            la      a2, rust_main
            or      a2, a2, t2
            jalr    a2                      // call rust_main
        ",
        boot_stack = sym BOOT_STACK,
        page_table = sym BOOT_PAGE_TABLE,
        virt_ram_offset = const VIRT_RAM_OFFSET,
        options(noreturn),
    )
}
```

=== 地址空间管理

Phonix 使用 RAII 机制管理地址空间，主要使用`MemorySpace` 和 `VmArea` 以及 `RangeMap`数据结构。

```rust
/// Virtual memory space for kernel and user.
pub struct MemorySpace {
    /// Page table of this memory space.
    page_table: PageTable,
    /// Map of `VmArea`s in this memory space.
    areas: RangeMap<VirtAddr, VmArea>,
}

/// A contiguous virtual memory area.
pub struct VmArea {
    /// Aligned `VirtAddr` range for the `VmArea`.
    range_va: Range<VirtAddr>,
    /// Hold pages with RAII.
    pub pages: BTreeMap<VirtPageNum, Arc<Page>>,
    /// Map permission of this area.
    pub map_perm: MapPerm,
    /// Type of this area.
    pub vma_type: VmAreaType,

    // For mmap.
    /// Mmap flags.
    pub mmap_flags: MmapFlags,
    /// The underlying file being mapped.
    pub backed_file: Option<Arc<dyn File>>,
    /// Start offset in the file.
    pub offset: usize,
}

/// A range map that stores range as key.
pub struct RangeMap<U: Ord + Copy + Add<usize>, V>(BTreeMap<U, Node<U, V>>);
```

== 缺页异常处理

Phoenix 目前能够利用缺页异常处理来实现写时复制（Copy on write）、懒分配（Lazy page allocation）以及用户地址检查机制和零拷贝技术。

当用户程序因缺页异常返回内核时，内核异常处理函数能够从 `stval` 寄存器读取异常发生的地址，并交给 `VmArea::handle_page_fault` 函数进行处理。


=== CoW 写时复制技术

在 `fork` 进程时，Phoenix 会将原`MemorySpace`中的除共享内存外每一个已分配页的PTE都删除写标志位，打上COW标志位，然后重新映射到页表中，并将。在用户向COW页写入时会触发缺页异常陷入内核，在`VmArea::handle_page_fault`函数中，内核会根据COW标志位转发给COW缺页异常处理函数，缺页异常处理函数会根据`Arc<Page>`的原子持有计数判断是否为最后一个持有者，如果不是最后一个持有者，会新分配一个页并复制原始页的数据并恢复写标志位重新映射，如果是最后一个持有者，直接恢复写标志位。

```rust
impl MemorySpace {
    /// Clone a same `MemorySpace` lazily.
    pub fn from_user_lazily(user_space: &mut Self) -> Self {
        let mut memory_space = Self::new_user();
        for (range, area) in user_space.areas().iter() {
            let mut new_area = area.clone();
            for vpn in area.range_vpn() {
                if let Some(page) = area.pages.get(&vpn) {
                    let pte = user_space
                        .page_table_mut()
                        .find_leaf_pte(vpn)
                        .unwrap();
                    let (pte_flags, ppn) = match area.vma_type {
                        VmAreaType::Shm => {
                            // no cow for shared memory
                            new_area.pages.insert(vpn, page.clone());
                            (pte.flags(), page.ppn())
                        }
                        _ => {
                            // copy on write
                            let mut new_flags = pte.flags() | PTEFlags::COW;
                            new_flags.remove(PTEFlags::W);
                            pte.set_flags(new_flags);
                            (new_flags, page.ppn())
                        }
                    };
                    memory_space.page_table_mut().map(vpn, ppn, pte_flags);
                } else {
                    // do nothing for lazy allocated area
                }
            }
            memory_space.push_vma_lazily(new_area);
        }
        memory_space
    }
}
```

=== 懒分配技术

懒分配技术主要用于堆栈分配以及mmap匿名映射或文件映射。在传统的内存分配方法中，操作系统在进程请求内存时会立即为其分配实际的物理内存。然而，这种方法在某些情况下可能导致资源的浪费，因为进程可能并不会立即使用全部分配的内存。

懒分配技术的核心思想是推迟实际物理内存的分配，直到进程真正访问到该内存区域。这样可以优化内存使用，提高系统性能。

对于内存的懒分配，比如堆栈分配，mmap匿名内存分配，Phoenix将许可分配的范围记录下来，但并不进行实际分配操作，当用户访问到许诺分配但未分配的页面时会触发缺页异常，缺页异常处理函数会进行实际的分配操作。


对于mmap文件的懒分配，Phoenix将其与页缓存机制深度融合，Phoenix 同样执行懒分配操作，当缺页异常时再从页缓存中获取页面。



=== 用户地址检查与零拷贝技术

在系统调用过程中，内核需要频繁与用户态指针指向的数据进行交互。在 Phonix 中，用户和内核共享地址空间，因此在访问用户态的内存时不需要同 xv6 那样通过软件查询页表，而是可以利用硬件页表机制直接解引用用户态指针。

然而，用户态指针并不总是有效的，有可能指向非法内存，出于安全性保证，内核需要能够捕获这种异常。在用户态下，无效指针解引用，或者向只读地址写入数据会触发缺页异常，陷入内核并执行相应缺页异常处理函数，通常，缺页异常处理函数会向程序发送 `SIGSEGV` 信号或者终止进程。而 Phoenix 内核态也需要解引用用户态的指针，因此内核也需要捕获并处理这种页错误。

我们参考了往届 MankorOS 队伍的做法，借助硬件 MMU 部件的帮助实现了高效的用户指针检查。该做法的基本思路是，先将内核的异常捕捉函数替换为“用户检查模式”下的函数，然后直接尝试向目标地址读取或写入一个字节。若是目标地址发生了缺页异常，则内核将表现得如同用户程序发生了一次异常一般，进入用户缺页异常处理程序进行处理。若处理成功或目标地址访问成功，便可假定当前整个页范围内都是合法的用户地址空间，否则用户指针便不合法。该处理方法相当于直接利用了硬件 MMU 来检查用户指针是否可读或可写，在用户指针正常时速度极快，同时还能完全复用用户缺页异常处理的代码来处理用户指针懒加载/CoW 的情况。

此外，Phoenix 基于此实现了用户态指针内容的零拷贝技术，即内核态不需要软件模拟地址翻译并复制用户态指针指向的数据到内核态，而是可以直接访问用户态指针，避免用户态数据到内核态数据的拷贝。用户态传入的指针经过检查后，会被转换成 `UserRef`、`UserMut` 和 `UserSlice` 对象。每一个对象都存放具体的指针，并保存了 `SumGuard` 以获取内核态访问用户地址空间的权限。Phoenix 充分利用了 Rust 提供的类型机制，为上述对象实现了 `deref` 方法，使得在不改变外部函数签名的情况下，依然能保持 `sstatus`寄存器`sum` 位开启，直接访问用户地址空间。这种实现不仅高效，还极大缩短了代码量。例如，`sys_read` 函数只需短短3行代码，不仅实现了用户地址空间检查，还能将文件内容零拷贝直接填充到用户提供的缓冲区。

```rust
/// User slice. Hold slice from `UserPtr` and a `SumGuard` to provide user
/// space access.
pub struct UserSlice<'a, T> {
    slice: &'a mut [T],
    _guard: SumGuard,
}

impl<'a, T> core::ops::DerefMut for UserSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slice
    }
}

pub async fn sys_read(
    &self,
    fd: usize,
    buf: UserWritePtr<u8>,
    count: usize,
) -> SyscallResult {
    let file = self.task.with_fd_table(|table| table.get_file(fd))?;
    let mut buf: UserSlice<'_, u8> = buf.into_mut_slice(&task, count)?;
    file.read(&mut buf).await
}
```

