//! Written by JD Lu
//! Refactored by Sean Lin
// 用于 loongarch icu 中断控制器的驱动
//
//                              +---------------------+
//                              | 可配置中断路由      |
//                              |                     |
// +-------------------------*  |                     |  +---------------------+
// | HT-1 INT7   --> 31      |  |                     |  | CORE 0              |
// | ......      --> ...     |  |                     |  |  IP0,IP1,IP2,IP3    |
// | HT-1 INT0   --> 24      |  |                     |  +---------------------+
// | HT-0 INT7   --> 23      |  |                     |  +---------------------+
// | ......      --> ...     |  |                     |  | CORE 1              |
// | HT-0 INT0   --> 16      |  |                     |  |  IP0,IP1,IP2,IP3    |
// | UART1       --> 15      |  |                     |  +---------------------+
// | Thsens      --> 14      |  |                     |  +---------------------+
// | SPI         --> 13      |  |                     |  | CORE 2              |
// | MC1         --> 12      |  |                     |  |  IP0,IP1,IP2,IP3    |
// | MC0         --> 11      |  |                     |  +---------------------+
// | UART0       --> 10      |  |                     |  +---------------------+
// | I2C1        --> 9       |  |                     |  | CORE 3              |
// | I2C0        --> 8       |  |                     |  |  IP0,IP1,IP2,IP3    |
// | GPIO31/23/15/7 -->7     |  |                     |  +---------------------+
// | GPIO30/22/14/6 -->6     |  |                     |
// | GPIO29/21/13/5 -->5     |  |                     |
// | GPIO28/20/12/4 -->4     |  |                     |
// | GPIO27/19/11/3/SC3 -->3 |  |                     |
// | GPIO26/18/10/2/SC2 -->2 |  |                     |
// | GPIO25/17/9/1/SC1 -->1  |  |                     |
// | GPIO24/16/8/0/SC0 -->0  |  |                     |
// +-------------------------+  +---------------------+

use bitflags::bitflags;
use core::ptr::{read_volatile, write_volatile};

// 根据龙芯 3A5000 用户手册，定义的寄存器偏移地址
// const OFFSET_INTISR: usize = 0x1420;
// const OFFSET_INTEN: usize = 0x1424;
// const OFFSET_INTENSET: usize = 0x1428;
// const OFFSET_INTENCLR: usize = 0x142c;
// const OFFSET_INTEDGE: usize = 0x1434;
// const OFFSET_CORE_INTISR: [usize; 4] = [0x1440, 0x1448, 0x1450, 0x1458];
// const OFFSET_ENTRY_BASE: usize = 0x1400;
const OFFSET_INTISR: usize = 0x20;
const OFFSET_INTEN: usize = 0x24;
const OFFSET_INTENSET: usize = 0x28;
const OFFSET_INTENCLR: usize = 0x2c;
const OFFSET_INTEDGE: usize = 0x34;
const OFFSET_CORE_INTISR: [usize; 4] = [0x40, 0x48, 0x50, 0x58];
const OFFSET_ENTRY_BASE: usize = 0x00;

/// 表示32个可用的中断源
/// Loongson 3A5000 用户手册, 表 11-1
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Interrupt {
    GpioSc0 = 0,
    GpioSc1 = 1,
    GpioSc2 = 2,
    GpioSc3 = 3,
    Gpio4 = 4,
    Gpio5 = 5,
    Gpio6 = 6,
    Gpio7 = 7,
    I2c0 = 8,
    I2c1 = 9,
    Uart0 = 10,
    Mc0 = 11,
    Mc1 = 12,
    Spi = 13,
    Thsens = 14,
    Uart1 = 15,
    Ht0Int0 = 16,
    Ht0Int1 = 17,
    Ht0Int2 = 18,
    Ht0Int3 = 19,
    Ht0Int4 = 20,
    Ht0Int5 = 21,
    Ht0Int6 = 22,
    Ht0Int7 = 23,
    Ht1Int0 = 24,
    Ht1Int1 = 25,
    Ht1Int2 = 26,
    Ht1Int3 = 27,
    Ht1Int4 = 28,
    Ht1Int5 = 29,
    Ht1Int6 = 30,
    Ht1Int7 = 31,
}

impl From<Interrupt> for u32 {
    fn from(irq: Interrupt) -> Self {
        irq as u32
    }
}

impl TryFrom<u32> for Interrupt {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 32 {
            // Safety: The check ensures that the value is a valid discriminant.
            // `Interrupt` is `#[repr(u32)]`, so this transmutation is safe.
            Ok(unsafe { core::mem::transmute(value) })
        } else {
            Err("Invalid interrupt number")
        }
    }
}

bitflags! {
    /// 中断路由寄存器
    /// loongson 3A5000 用户手册 表 11-3
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
    pub struct ConfigVector: u8 {
        // 路由到的 cpu 编号
        const CORE_0 = 1 << 0;
        const CORE_1 = 1 << 1;
        const CORE_2 = 1 << 2;
        const CORE_3 = 1 << 3;
        // 路由到的引脚编号
        const PIN_0 = 1 << 4;
        const PIN_1 = 1 << 5;
        const PIN_2 = 1 << 6;
        const PIN_3 = 1 << 7;
    }
}

impl ConfigVector {
    /// 创建一个新的路由配置
    /// `core_id`: 0-3
    /// `pin_id`: 0-3
    pub fn new(core_id: u8, pin_id: u8) -> Option<Self> {
        if core_id > 3 || pin_id > 3 {
            return None;
        }
        let core_flag = 1 << core_id;
        let pin_flag = 1 << (pin_id + 4);
        Some(Self::from_bits_truncate(core_flag | pin_flag))
    }
}

/// 龙芯中断控制器单元 (ICU) 驱动
pub struct LocalIOIntController {
    base_addr: usize,
}

impl LocalIOIntController {
    /// 创建一个新的 ICU 驱动实例
    /// # Safety
    /// 调用者必须确保 `base_addr` 是 ICU 控制器的正确物理地址,
    /// 并且这是唯一的实例。
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    /// 使能一个特定的中断源
    pub fn enable(&self, irq: Interrupt) {
        let addr = (self.base_addr + OFFSET_INTENSET) as *mut u32;
        let irq_mask = 1 << (irq as u32);
        unsafe { write_volatile(addr, irq_mask) };
    }

    /// 禁用一个特定的中断源并清除其中断挂起状态
    pub fn disable(&self, irq: Interrupt) {
        let addr = (self.base_addr + OFFSET_INTENCLR) as *mut u32;
        let irq_mask = 1 << (irq as u32);
        unsafe { write_volatile(addr, irq_mask) };
    }

    /// 检查一个中断源当前是否被使能
    pub fn is_enabled(&self, irq: Interrupt) -> bool {
        let addr = (self.base_addr + OFFSET_INTEN) as *const u32;
        let irq_mask = 1 << (irq as u32);
        let enabled_mask = unsafe { read_volatile(addr) };
        (enabled_mask & irq_mask) != 0
    }

    /// 设置中断源的触发模式
    /// `edge_triggered`: true 表示边沿触发, false 表示电平触发
    pub fn set_trigger_mode(&self, irq: Interrupt, edge_triggered: bool) {
        let addr = (self.base_addr + OFFSET_INTEDGE) as *mut u32;
        let irq_mask = 1 << (irq as u32);
        unsafe {
            let current_modes = read_volatile(addr);
            let new_modes = if edge_triggered {
                current_modes | irq_mask
            } else {
                current_modes & !irq_mask
            };
            write_volatile(addr, new_modes);
        }
    }

    /// 获取中断源的触发模式
    /// 返回 `true` 如果是边沿触发, `false` 如果是电平触发
    pub fn get_trigger_mode(&self, irq: Interrupt) -> bool {
        let addr = (self.base_addr + OFFSET_INTEDGE) as *const u32;
        let irq_mask = 1 << (irq as u32);
        let modes = unsafe { read_volatile(addr) };
        (modes & irq_mask) != 0
    }

    /// 检查一个中断是否全局挂起
    pub fn is_pending(&self, irq: Interrupt) -> bool {
        let addr = (self.base_addr + OFFSET_INTISR) as *const u32;
        let irq_mask = 1 << (irq as u32);
        let pending_mask = unsafe { read_volatile(addr) };
        (pending_mask & irq_mask) != 0
    }

    /// 获取指定核心上所有挂起中断的32位掩码
    pub fn get_core_pending_mask(&self, core_id: u8) -> Option<u32> {
        if core_id > 3 {
            return None;
        }
        let addr = (self.base_addr + OFFSET_CORE_INTISR[core_id as usize]) as *const u32;
        Some(unsafe { read_volatile(addr) })
    }

    /// 将一个中断源路由到特定的核心和引脚
    pub fn route(&self, irq: Interrupt, config: ConfigVector) {
        let entry_offset = irq as u32;
        let addr = (self.base_addr + OFFSET_ENTRY_BASE + entry_offset as usize) as *mut u8;
        unsafe { write_volatile(addr, config.bits()) };
    }

    /// 获取一个中断源当前的路由配置
    pub fn get_route(&self, irq: Interrupt) -> ConfigVector {
        let entry_offset = irq as u32;
        let addr = (self.base_addr + OFFSET_ENTRY_BASE + entry_offset as usize) as *const u8;
        let bits = unsafe { read_volatile(addr) };
        ConfigVector::from_bits_truncate(bits)
    }
}

/// 这是一个测试函数，可以从内核的主初始化函数中调用。
/// 它假设 `println!` 宏可用于输出。
/// 它也假设它正在一个龙芯系统上运行，这些操作是有效的。
/// 如果在其他架构上运行，此代码将会 panic。
///
/// # Safety
/// 此函数直接对硬件寄存器进行内存访问，只应在适当的上下文
/// (例如，在目标硬件上的内核初始化期间) 调用。`base_addr` 必须是有效的。
#[allow(dead_code)]
pub unsafe fn test_loongarch_icu(base_addr: usize) {
    // Assuming println! is available in the kernel.
    // If not, status should be reported in another way.
    println!("--- Starting LoongArch ICU driver test ---");

    let icu = LocalIOIntController::new(base_addr);
    let test_irq = Interrupt::Uart0;

    println!("Testing IRQ: {:?}", test_irq);

    // 1. Test enable/disable
    println!("1. Testing enable/disable...");
    icu.disable(test_irq);
    assert!(!icu.is_enabled(test_irq), "Interrupt should be disabled");
    println!("  - Disable OK.");
    icu.enable(test_irq);
    assert!(icu.is_enabled(test_irq), "Interrupt should be enabled");
    println!("  - Enable OK.");
    icu.disable(test_irq);
    assert!(
        !icu.is_enabled(test_irq),
        "Interrupt should be disabled again"
    );
    println!("  - Disable again OK.");

    // 2. Test trigger mode
    println!("2. Testing trigger mode...");
    icu.set_trigger_mode(test_irq, false); // Level-triggered
    assert!(!icu.get_trigger_mode(test_irq), "Should be level-triggered");
    println!("  - Set to level-triggered OK.");
    icu.set_trigger_mode(test_irq, true); // Edge-triggered
    assert!(icu.get_trigger_mode(test_irq), "Should be edge-triggered");
    println!("  - Set to edge-triggered OK.");

    // 3. Test routing
    println!("3. Testing routing...");
    let route_config = ConfigVector::new(2, 3).expect("Valid config"); // Core 2, Pin 3
    icu.route(test_irq, route_config);
    let read_config = icu.get_route(test_irq);
    assert_eq!(route_config, read_config, "Route config should match");
    println!(
        "  - Route to Core 2, Pin 3 OK. Read back: {:?}",
        read_config
    );

    let route_config_2 = ConfigVector::new(0, 0).expect("Valid config"); // Core 0, Pin 0
    icu.route(test_irq, route_config_2);
    let read_config_2 = icu.get_route(test_irq);
    assert_eq!(route_config_2, read_config_2, "Route config 2 should match");
    println!(
        "  - Route to Core 0, Pin 0 OK. Read back: {:?}",
        read_config_2
    );

    // Cleanup
    icu.disable(test_irq);
    icu.set_trigger_mode(test_irq, false); // Default to level-triggered
    icu.route(test_irq, ConfigVector::default()); // Default route

    println!("--- LoongArch ICU driver test finished successfully ---");
}
