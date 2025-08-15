# LoongArch架构下AddressNotAligned异常的深入解析

本文将基于 `polyhal-trap` 模块中的代码，详细解释LoongArch架构下`AddressNotAligned`（地址未对齐）异常的触发原因，以及操作系统在trap中断处理流程中解决该异常的精妙方法。

## 一、AddressNotAligned 异常的触发原因

在计算机体系结构中，内存对齐是一种设计要求，旨在优化CPU访问内存的性能。LoongArch架构也不例外。

**核心原因：**

LoongArch的加载（Load）和存储（Store）指令，如 `LD.W` (加载字, 4字节)、`LD.D` (加载双字, 8字节)、`ST.W` (存储字)、`ST.D` (存储双字) 等，在硬件层面要求其访问的内存地址必须是其访问数据大小的整数倍。

- **访问字 (Word, 4字节):** 内存地址必须能被4整除。
- **访问双字 (Double Word, 8字节):** 内存地址必须能被8整除。
- **访问半字 (Half Word, 2字节):** 内存地址必须能被2整除。

当CPU执行一条加载或存储指令，但目标内存地址不满足上述对齐要求时，硬件会无法完成这次内存访问。此时，它不会继续执行，而是会立即触发一个**硬件异常**，并将异常类型设置为 `AddressNotAligned`。随后，CPU会跳转到操作系统预设的异常处理入口点，将控制权交给操作系统内核。

**举例：**
假设程序执行 `ld.d $r4, $r5, 0` 指令，该指令意图从寄存器 `$r5` 中的地址加载一个8字节的数据到寄存器 `$r4`。如果此时 `$r5` 寄存器中的地址是 `0x10007`，由于 `0x10007` 不能被8整除，硬件就会触发 `AddressNotAligned` 异常。

## 二、操作系统在Trap中的解决方法：指令仿真

面对硬件抛出的 `AddressNotAligned` 异常，操作系统可以选择直接终止引发异常的程序（即所谓的Panic或Crash），但这会牺牲程序的健壮性。一个更优雅的解决方案是**在软件层面模拟（Emulate）**这条引发异常的指令，让程序能够透明地继续执行，就好像硬件原生支持非对齐访问一样。

`polyhal-trap` 模块正是采用了这种指令仿真的方法。整个处理流程如下：

### 1. 异常捕获与分发

当异常发生时，CPU跳转到 `trap_vector_base`，经过一系列上下文保存操作后，最终调用 `loongarch64_trap_handler` 函数。

```rust
// @polyhal/polyhal-trap/src/trap/loongarch64.rs

fn loongarch64_trap_handler(tf: &mut TrapFrame) -> TrapType {
    let estat = estat::read();
    let trap_type = match estat.cause() {
        // ... 其他异常类型
        Trap::Exception(Exception::AddressNotAligned) => {
            // 关键：当中断原因是地址未对齐时，调用仿真函数
            unsafe { emulate_load_store_insn(tf) }
            TrapType::Unknown
        }
        // ... 其他异常类型
    };
    // ...
    trap_type
}
```
该函数通过读取 `estat` 寄存器来判断异常原因。当确定是 `AddressNotAligned` 异常时，它会调用核心处理函数 `emulate_load_store_insn`。

### 2. 指令仿真核心逻辑 (`emulate_load_store_insn`)

这个函数位于 `unaligned.rs` 文件中，是解决非对齐访问问题的核心。它的工作可以分为以下几个步骤：

#### a. 获取异常现场信息

函数首先需要知道是哪条指令、在哪个地址上发生了问题。

```rust
// @polyhal/polyhal-trap/src/trap/loongarch64/unaligned.rs

pub unsafe fn emulate_load_store_insn(pt_regs: &mut TrapFrame) {
    let la_inst: u32; // 用于存储导致异常的指令
    let addr: u64;    // 导致异常的内存地址
    let rd: usize;    // 目标/源寄存器索引

    // 1. 从 TrapFrame 中获取程序计数器(era)，也就是指令的地址
    //    然后从该地址加载指令本身。
    core::arch::asm!(
        "ld.w {val}, {addr}, 0 ",
         addr = in(reg) pt_regs.era as u64,
         val = out(reg) la_inst,
    )

    // 2. 从 badv (Bad Virtual Address) 寄存器中读取导致错误的内存地址
    addr = badv::read().vaddr() as u64;

    // 3. 从指令的二进制编码中解析出目标寄存器 rd
    rd = (la_inst & 0x1f) as usize;
    // ...
}
```

#### b. 解码并识别指令类型

获取到指令的二进制码 `la_inst` 后，代码通过位运算和与预定义的指令操作码（`OP`）常量进行比较，来判断这是一条什么类型的指令（加载/存储、双字/字/半字、有符号/无符号）。

```rust
// @polyhal/polyhal-trap/src/trap/loongarch64/unaligned.rs

// 判断是否是加载双字(LDD)、加载指针双字(LDPTRD)或变址加载双字(LDXD)
if (la_inst >> 22) == LDD_OP || (la_inst >> 24) == LDPTRD_OP || (la_inst >> 15) == LDXD_OP {
    // ...
} 
// 判断是否是加载字(LDW)、加载指针字(LDPTRW)或变址加载字(LDXW)
else if (la_inst >> 22) == LDW_OP
    || (la_inst >> 24) == LDPTRW_OP
    || (la_inst >> 15) == LDXW_OP
{
    // ...
} 
// 判断是否是存储双字(STD)、存储指针双字(STPTRD)或变址存储双字(STXD)
else if (la_inst >> 22) == STD_OP
    || (la_inst >> 24) == STPTRD_OP
    || (la_inst >> 15) == STXD_OP
{
    // ...
} 
// ... 其他指令类型的判断
```

#### c. 执行软件模拟的内存访问

识别出指令后，操作系统接管硬件的工作，在软件层面完成内存访问。这是通过 `unaligned_read` 和 `unaligned_write` 这两个底层汇编函数实现的。

- **对于加载指令 (Load):**
  ```rust
  // @polyhal/polyhal-trap/src/trap/loongarch64/unaligned.rs
  
  // 以加载双字为例
  res = unaligned_read(addr, &mut value, 8, 1); // 软件读取8个字节
  if res < 0 {
      panic!("Address Error @ {:#x}", addr)
  }
  // 将读取到的值写入保存在TrapFrame中的寄存器副本
  pt_regs.regs[rd] = value as usize; 
  ```
  `unaligned_read` 函数通过**逐字节读取**的方式，从非对齐的内存地址 `addr` 中读取指定数量（如8字节）的数据，然后在内部将这些字节拼接成一个完整的值。最后，`emulate_load_store_insn` 将这个拼接好的值更新到 `TrapFrame` 中对应的寄存器 `rd` 的位置。

- **对于存储指令 (Store):**
  ```rust
  // @polyhal/polyhal-trap/src/trap/loongarch64/unaligned.rs

  // 以存储双字为例
  value = pt_regs.regs[rd] as u64; // 从TrapFrame中获取要存储的值
  res = unaligned_write(addr, value, 8); // 软件写入8个字节
  ```
  `unaligned_write` 函数将要存储的值 `value` **拆分成单个字节**，然后逐字节地写入到非对齐的内存地址 `addr`。

#### d. 更新程序计数器并返回

指令仿真成功后，必须手动更新程序计数器（`era`），让它指向下一条指令，否则CPU从异常返回后会再次执行这条导致异常的指令，从而陷入死循环。

```rust
// @polyhal/polyhal-trap/src/trap/loongarch64/unaligned.rs

// LoongArch指令长度为4字节，所以将era加4
pt_regs.era += 4;
```

### 3. 返回用户态

当 `emulate_load_store_insn` 执行完毕后，控制权返回到 `loongarch64_trap_handler`，再经过一系列上下文恢复操作（`LOAD_REGS` 和 `ertn`），最终返回到用户程序。此时，对于用户程序而言，它感觉到的仅仅是那条非对齐访问指令的执行时间稍长了一些，而并不知道期间发生了一次复杂的异常处理和指令仿真过程。

## 总结

`AddressNotAligned` 异常是LoongArch硬件对非对齐内存访问的正常响应。`polyhal-trap` 模块通过捕获此异常，并在内核中对引发异常的指令进行**解码、分析和软件仿真**，巧妙地解决了这一问题。这种方法以一定的性能开销（陷入内核、软件模拟）为代价，换取了程序的**健壮性和兼容性**，使得应用程序开发者无需过分关注内存对齐的细节，从而简化了编程。