#import "../components/prelude.typ": *

= 中断控制器

中断控制器用于管理硬件中断请求（IRQ）

Del0n1x 基于 RISCV qemu、LoongArch qemu、龙芯2k1000、StarFiveVison平台编写其各自的中断控制器

== LoongArch 架构



目前基于 LoongArch 的处理器只能和 LS7A 芯片组工作。LoongArch计算机
中的中断控制器（即IRQ芯片）包括CPUINTC（CPU Core Interrupt Controller）、LIOINTC（
Legacy I/O Interrupt Controller）、EIOINTC（Extended I/O Interrupt Controller）、
HTVECINTC（Hyper-Transport Vector Interrupt Controller）、PCH-PIC（LS7A芯片组的主中
断控制器）、PCH-LPC（LS7A芯片组的LPC中断控制器）和PCH-MSI（MSI中断控制器）。

我们基于 LoongArch 平台的中断模型，对 LIOINTC、EIOINTC、PCH-PIC 进行抽象与配置

LoongArch 架构下 qemu 平台与 龙芯 2k1000 平台采用不同的中断控制模型

#figure(
  image("assets/loongarch_2k1000_int.png"),
  caption: [龙芯 2k1000 中断模型],
  supplement: [图]
)

在这一模型中，核间中断（IPI）和 CPU 本地时钟中断直接发送到 CPUINTC，串口（UARTs）中断发送到 LIOINTC，而其他所有设备的中断分别发送到所连接的PCH-PIC/PCH-LPC/PCH-MSI，然后被HTVECINTC统一收集，再发送到LIOINTC，最后到达CPUINTC

#figure(
  image("assets/loongarch_qemu_int.png"),
  caption: [龙芯 qemu 中断模型],
  supplement: [图]
)

在这种模型里面, IPI(Inter-Processor Interrupt) 和CPU本地时钟中断直接发送到CPUINTC,
CPU串口 (UARTs) 中断发送到PCH-PIC, 而其他所有设备的中断则分别发送到所连接的PCH_PIC/
PCH-MSI, 然后V-EIOINTC统一收集，再直接到达CPUINTC


LIOINTC 在设备树中对应

#code-figure(
  ```raw
interrupt-controller@1fe01400 {
		compatible = "loongson,2k1000-icu";
		interrupt-controller;
		#interrupt-cells = <0x01>;
		reg = <0x00 0x1fe01400 0x00 0x40 0x00 0x1fe01040 0x00 0x10>;
		interrupt-parent = <0x01>;
		interrupt-names = "cascade";
		interrupts = <0x03>;
		phandle = <0x06>;
	};
```,
  caption: [LIOINTC],
  label-name: "LIOINTC 在设备树中的定义"
)



在 LIOINTC 中定义 32个中断源对应的路由向量寄存器和若干控制寄存器。配置指定中断源需要首先配置 INTENSET 寄存器使能指定的中断源，然后定义路由向量，将路由向量写入到对应中断源的寄存器当中

#code-figure(
  ```rs
// 定义 LIOINTC 中断控制器
let icu = LocalIOIntController::new(base_addr);
// 定义中断源：Uart
let test_irq = Interrupt::Uart0.into();
// 禁用中断源
icu.disable(test_irq);
// 使能中断源
icu.enable(test_irq);
// 设置路由向量，即路由到核心 2 的 3号外部中断源
let route_config = ConfigVector::new(2, 3).expect("Valid config"); // Core 2, Pin 3
// 将路由向量写入到对应的中断源的路由向量寄存器当中
icu.route(test_irq, route_config);
```,
  caption: [LIOINTC],
  label-name: "LIOINTC 配置终端源与中断路由样例"
)

EIOINTC 和 PCH-PIC 配置方式和 LIOINTC 类似








