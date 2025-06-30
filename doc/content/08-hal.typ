#import "../components/prelude.typ": *

= 硬件抽象层

== 硬件抽象层总览

为支持多平台运行与测试，Del0n1x 在内核中实现了功能基本完整的硬件抽象层（Hardware Abstraction Layer，HAL），可以运行于 RISC-V64 和 LoongArch64 两种指令集架构的 QEMU 平台。Del0n1x的内核代码全部基于硬件抽象层开发，屏蔽了架构细节和平台差异，具有更好的兼容性和可移植性。

#figure(
  image("assets/硬件抽象层示意图.png"),
  caption: [硬件抽象层示意图],
  supplement: [图]
)


== 处理器访问接口

// 特殊通用寄存器读写、控制状态寄存器读写、特殊指令执行
Del0n1x 在`riscv`和`loongarch`外部库的帮助下，对 RISC-V 和 LoongArch 两种架构下的处理器访问实现了统一抽象。在内核代码中，对处理器及相关资源的访问包括以下几种类型：

#list(
    [读写通用寄存器],
    [访问控制状态寄存器],
    [执行特殊的控制指令（如内核刷表指令）],
    indent: 2em
)
// - 读写通用寄存器 
// - 访问控制状态寄存器
// - 执行特殊的控制指令（如内核刷表指令）

#h(2em)在 Del0n1x 中，这些操作只需要调用统一的接口即可完成，具体实现细节位于 HAL 中，便于构建架构无关的内核代码。

== 内核入口例程

RISC-V 和 LoongArch 架构在内核启动上的细节有所异同。LoongArch 架构启动时需要配置直接映射地址翻译模式，需要为分页地址翻译模式设置高半空间、低半空间两个页表 token ，还需要设置 TLB 重填异常入口，这些都是 RISC-V 架构下内核启动所不需要的步骤。Del0n1x 的硬件抽象层将初始化时的架构相关部分抽象为`arch_init()`例程，供内核初始化时调用，为内核入口的规范化提供支持。

// 【代码，arch_init重构完毕后填写】
#code-figure(
  ```rust
// loongarch
pub fn arch_init() {
    mmu_init();
    euen::set_fpe(true);
    tlb_init(tlb_fill as usize);
}
```,
  caption: [arch_init 例程],
  label-name: "arch_init"
)


== 内存管理单元与地址空间

=== 物理内存

RISC-V 和 LoongArch 的 QEMU virt 平台的物理编址方式和布局存在很大区别。以下分别为两个平台的物理地址布局：

#figure(
  image("assets/物理地址布局.png"),
  caption: [QEMU virt 物理地址布局],
  supplement: [图]
)

RISC-V QEMU virt 平台将 RAM 编码于`0x8000_0000`以上的物理地址空间。而 LoongArch QEMU virt 平台则将 RAM 切分为`lowram`和`highram`两个部分，其中`lowram`位于`0x1000_0000`以下的物理地址空间，`highram`位于`0x9000_0000`以上的物理地址空间。Del0n1x 将内核镜像加载于`lowram`中。

=== 分页地址翻译模式

// rv完整 la两半
Del0n1x 使用 SV39 分页地址翻译模式。在不考虑直接映射窗口的前提下， Del0n1x 将完整的39位虚拟地址空间分为高半部分和低半部分，高半部分的地址范围为`0xffff_ffc0_0000_0000~0xffff_ffff_ffff_ffff`，低半部分的地址范围为`0x0000_0000_0000_0000~0x0000_003f_ffff_ffff`。在分页地址翻译模式中，地址格式不满足 39 位整数的符号拓展形式的，将被视为非法地址。

RISC-V 架构使用控制状态寄存器 SATP 控制分页模式类型（如SV39）并存储页表根目录的物理页号。Del0n1x 为每一个进程创建一张页表，并为其应用整个虚拟地址空间内的全部映射。当地址空间切换时，通过对 SATP 的修改，即可达到切换页表的目的。

LoongArch 架构的情况有所不同。LoongArch 架构使用 CSR.CRMD 控制状态寄存器的 PG 位开关分页地址翻译模式，通过设置 CSR.PWCL 和 CSR.PWCH 控制状态寄存器中的`Dir{}_base`、`Dir{}_width`等部分手动配置分页模式细节。同时，LoongArch 架构下虚拟地址空间低半部分和高半部分的遍历应用不同的页表，其根目录地址分别经由 CSR.PGDL 和 CSR.PGDH 控制状态寄存器设置。这意味着 LoongArch 架构下一个进程需要创建两张页表，分别映射虚拟地址空间的两半。当地址空间切换时，需要按需修改 CSR.PGDL 或 CSR.PGDH ，从而灵活保留不需要切换的部分。

Del0n1x 在硬件抽象层中为两种架构实现了分页地址翻译模式的初始化，为 LoongArch 手动配置了 SV39 分页模式。由于 Del0n1x 使用虚拟地址空间的高半部分作为通用的内核地址空间，借助 LoongArch 架构的“双目录”设计，在地址空间切换时只需修改 CSR.PGDL。

#code-figure(
```rust
pub fn mmu_init() {
    // 设置页表项长度
    pwcl::set_pte_width(8); 
    // 设置页表第三级目录的索引位位置和长度
    pwcl::set_ptbase(PAGE_SIZE_SHIFT);
    pwcl::set_ptwidth(PAGE_SIZE_SHIFT - 3);
    // 设置页表第二级目录的索引位位置和长度
    pwcl::set_dir1_base(PAGE_SIZE_SHIFT + PAGE_SIZE_SHIFT - 3);
    pwcl::set_dir1_width(PAGE_SIZE_SHIFT - 3);
    // 设置页表根目录的索引位位置和长度
    pwch::set_dir3_base(PAGE_SIZE_SHIFT + PAGE_SIZE_SHIFT - 3 
      + PAGE_SIZE_SHIFT - 3);
    pwch::set_dir3_width(PAGE_SIZE_SHIFT - 3);
}
```,
  caption: [LoongArch SV39 分页地址翻译模式初始化],
  label-name: "la-mmu-init"
)


=== 页表

Del0n1x 支持两种架构下的多级页表。在硬件抽象层中，Del0n1x 为两种架构分别实现了对页表项（Page Table Entry，PTE）的封装，可以便捷地访问 PTE 中存储的物理页号和各标志位。Del0n1x 使用 Rust 宏为 PTE 标志位定义了统一的`checker`和`setter`方法，用于在内核代码中灵活修改 PTE 标志位。

#code-figure(
```rust
bitflags! {
    pub struct PTEFlags: usize {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        ... // 篇幅需要，省略
        const COW = 1 << 8;
    }
}
impl PTEFlags {
    impl_flag_checker!(
        pub U,
        pub V,
        ... // 篇幅需要，省略
    );// 为标志位FLAG实现is_[FLAG](&self)方法
    impl_flag_setter!(
        pub U,
        pub V,
        ... // 篇幅需要，省略
    );// 为标志位FLAG实现set_[FLAG](&mut self, bool)方法
}
``` ,
  caption: [RISC-V PTEFlags 实现],
  label-name: "riscv-pteflags-impl"
)


#h(2em)Del0n1x 的设计中，LoongArch 架构的内核地址空间需要单独使用一张页表，而RISC-V架构的内核地址空间则与用户地址空间共用同一张。Del0n1x 在硬件抽象层中为两个架构均映射一张内核页表，区别在于 LoongArch 架构下该页表会被写入 CSR.PGDH 并永不切换，而 RISC-V 架构下该页表只作为一个映射用户地址空间时的模板。当创建新的进程页表时，内核页表中根目录中映射地址空间高半部分的页表项将被复制到新的页表中，供内核态访问。

=== 直接映射窗口

LoongArch 架构支持直接映射地址翻译模式，该模式下允许通过修改 CSR.DMW0 \~ CSR.DMW3 控制状态寄存器配置至多4个直接映射窗口。当虚拟地址的高4位（在LoongArch64中为[63:60]位）恰好与某个直接映射窗口的高4位相同时，虚拟地址将被直接映射为其低 PALEN 位的物理地址（PALEN为机器有效物理地址长度）。使用`cpucfg`指令可以查明，LoongArch QEMU virt平台下 PALEN=48，故直接映射窗口将`0xW000_xxxx_xxxx_xxxx`范围内的所有虚拟地址映射为`0x0000_xxxx_xxxx_xxxx`。通过修改 CSR.DMWx 还可配置直接映射窗口的允许访问特权级、存储访问类型。

Del0n1x 使用`0x8000_xxxx_xxxx_xxxx`和`0x9000_xxxx_xxxx_xxxx`两个直接映射窗口，均限定内核特权级（PLV0）使用，分别用于设备访问和物理内存访问。

=== TLB重填

LoongArch 架构使用软件管理 TLB。当发生 TLB 中没有匹配项时，将触发 TLB 重填异常，跳转到内核设置的 TLB 重填入口执行软件重填。现阶段 Del0n1x 使用了往届优秀作品 NPUCore-IMPACT 编写的 TLB 重填代码。


#pagebreak()  // 强制分页