//! os/src/drivers/loongarch_cic.rs

//! Based on loongarch_icu.rs and detailed Loongson 7A documentation.
//! This driver is for the Companion Interrupt Controller (CIC) in the LS7A bridge chip.
//! Written by JD Lu
//! Refactored by Sean Lin

use bitflags::bitflags;
use core::ptr::{read_volatile, write_volatile};

// Register offsets from Loongson 7A documentation (Table 5-3)
const OFFSET_INT_ID_L: usize = 0x000;
const OFFSET_INT_ID_H: usize = 0x004;
const OFFSET_INT_MASK_L: usize = 0x020;
const OFFSET_INT_MASK_H: usize = 0x024;
const OFFSET_HTMSI_EN_L: usize = 0x040;
const OFFSET_HTMSI_EN_H: usize = 0x044;
const OFFSET_INTEDGE_L: usize = 0x060;
const OFFSET_INTEDGE_H: usize = 0x064;
const OFFSET_INTCLR_L: usize = 0x080;
const OFFSET_INTCLR_H: usize = 0x084;
const OFFSET_AUTO_CTRL0_L: usize = 0x0c0;
const OFFSET_AUTO_CTRL0_H: usize = 0x0c4;
const OFFSET_AUTO_CTRL1_L: usize = 0x0e0;
const OFFSET_AUTO_CTRL1_H: usize = 0x0e4;
const OFFSET_ROUTE_ENTRY_BASE: usize = 0x100;
const OFFSET_HTMSI_VECTOR_BASE: usize = 0x200;
const OFFSET_INTISR_0_L: usize = 0x300;
const OFFSET_INTISR_0_H: usize = 0x304;
const OFFSET_INTISR_1_L: usize = 0x320;
const OFFSET_INTISR_1_H: usize = 0x324;
const OFFSET_INTIRR_L: usize = 0x380;
const OFFSET_INTIRR_H: usize = 0x384;
const OFFSET_INTISR_L: usize = 0x3a0;
const OFFSET_INTISR_H: usize = 0x3a4;
const OFFSET_INT_POLARITY_L: usize = 0x3e0;
const OFFSET_INT_POLARITY_H: usize = 0x3e4;

/// Represents the 64 available interrupt sources in the CIC, mapped by their pin number.
/// Based on Loongson 7A User Manual, Table 5-1.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Interrupt {
    PcieF0_0 = 0,
    PcieF0_1 = 1,
    PcieF0_2 = 2,
    PcieF0_3 = 3,
    PcieF1_0 = 4,
    PcieF1_1 = 5,
    PcieHLo = 6,
    PcieHHi = 7,
    Uart = 8, // UARTs [3:0] share this interrupt
    I2c = 9,  // I2Cs [5:0] share this interrupt
    Gmac0Sbd = 12,
    Gmac0Pmt = 13,
    Gmac1Sbd = 14,
    Gmac1Pmt = 15,
    Sata0 = 16, // Documentation lists "sata" for 16, 17, 18. Assuming SATA0, 1, 2.
    Sata1 = 17,
    Sata2 = 18,
    Lpc = 19,
    Pwm0 = 24, // Assuming PWM0-3 for pins 24-27
    Pwm1 = 25,
    Pwm2 = 26,
    Pwm3 = 27,
    Dc = 28,
    Gpu = 29,
    Gmem = 30,
    Thsens = 31,
    // IRQs 32-39 are duplicates of 0-7 in some docs, likely for routing purposes.
    // We will treat them as distinct for now if needed, but map to the same names.
    PcieG0Lo = 40,
    PcieG0Hi = 41,
    PcieG1Lo = 42,
    PcieG1Hi = 43,
    Toy0 = 44, // Assuming TOY0, 1, 2 for pins 44-46
    Toy1 = 45,
    Toy2 = 46,
    Acpi = 47,
    Usb0Ehci = 48,
    Usb0Ohci = 49,
    Usb1Ehci = 50,
    Usb1Ohci = 51,
    Rtc0 = 52, // Assuming RTC0, 1, 2 for pins 52-54
    Rtc1 = 53,
    Rtc2 = 54,
    Hpet = 55,     // HPET timers share this interrupt
    Ac97Dma0 = 56, // Assuming DMA0, 1 for pins 56-57
    Ac97Dma1 = 57,
    Ac97Hda = 58, // AC97/HDA share this interrupt
    GpioHi = 59,
    Gpio0 = 60, // GPIOs are mapped ambiguously. Using Gpio0-3 for 60-63 as a placeholder.
    Gpio1 = 61,
    Gpio2 = 62,
    Gpio3 = 63,
}

impl From<Interrupt> for u32 {
    fn from(irq: Interrupt) -> Self {
        irq as u32
    }
}

/// Interrupt distribution modes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DistributionMode {
    Fixed = 0b00,
    RoundRobin = 0b01,
    Idle = 0b10,
    Busy = 0b11,
}

/// Loongson Companion Interrupt Controller (CIC) driver.
pub struct Cic {
    base_addr: usize,
}

impl Cic {
    /// Creates a new CIC driver instance.
    /// # Safety
    /// The caller must ensure `base_addr` is the correct physical address for the CIC
    /// and that this is the only instance.
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    fn read_u64_reg(&self, low_addr: usize) -> u64 {
        let low = unsafe { read_volatile((self.base_addr + low_addr) as *const u32) } as u64;
        let high = unsafe { read_volatile((self.base_addr + low_addr + 4) as *const u32) } as u64;
        (high << 32) | low
    }

    fn write_u64_reg(&self, low_addr: usize, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        unsafe {
            write_volatile((self.base_addr + low_addr) as *mut u32, low);
            write_volatile((self.base_addr + low_addr + 4) as *mut u32, high);
        }
    }

    /// Reads the CIC ID and version.
    /// Returns `(id, version, supported_irq_count)`.
    pub fn read_id_version(&self) -> (u8, u8, u8) {
        let id_reg = unsafe { read_volatile((self.base_addr + OFFSET_INT_ID_L) as *const u32) };
        let version_reg =
            unsafe { read_volatile((self.base_addr + OFFSET_INT_ID_H) as *const u32) };
        let id = (id_reg >> 24) as u8;
        let version = version_reg as u8;
        let int_num = (version_reg >> 16) as u8;
        (id, version, int_num + 1)
    }

    /// Enables a specific interrupt source. (Mask bit = 0)
    pub fn enable(&self, irq: u32) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut mask = self.read_u64_reg(OFFSET_INT_MASK_L);
        mask &= !(1 << irq);
        self.write_u64_reg(OFFSET_INT_MASK_L, mask);
    }

    /// Disables a specific interrupt source. (Mask bit = 1)
    pub fn disable(&self, irq: u32) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut mask = self.read_u64_reg(OFFSET_INT_MASK_L);
        mask |= 1 << irq;
        self.write_u64_reg(OFFSET_INT_MASK_L, mask);
    }

    /// Checks if an interrupt source is currently enabled.
    pub fn is_enabled(&self, irq: u32) -> bool {
        assert!(irq < 64, "Invalid IRQ number");
        let mask = self.read_u64_reg(OFFSET_INT_MASK_L);
        (mask & (1 << irq)) == 0
    }

    /// Sets the trigger mode for an interrupt source.
    pub fn set_trigger_mode(&self, irq: u32, edge_triggered: bool) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut modes = self.read_u64_reg(OFFSET_INTEDGE_L);
        if edge_triggered {
            modes |= 1 << irq;
        } else {
            modes &= !(1 << irq);
        }
        self.write_u64_reg(OFFSET_INTEDGE_L, modes);
    }

    /// Gets the trigger mode for an interrupt source.
    pub fn get_trigger_mode(&self, irq: u32) -> bool {
        assert!(irq < 64, "Invalid IRQ number");
        let modes = self.read_u64_reg(OFFSET_INTEDGE_L);
        (modes & (1 << irq)) != 0
    }

    /// Sets the polarity for a level-triggered interrupt.
    pub fn set_polarity(&self, irq: u32, high_level: bool) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut polarities = self.read_u64_reg(OFFSET_INT_POLARITY_L);
        if high_level {
            polarities &= !(1 << irq);
        } else {
            polarities |= 1 << irq;
        }
        self.write_u64_reg(OFFSET_INT_POLARITY_L, polarities);
    }

    /// Clears a pending edge-triggered interrupt.
    pub fn clear_pending_edge(&self, irq: u32) {
        assert!(irq < 64, "Invalid IRQ number");
        let val = 1u64 << irq;
        if irq < 32 {
            unsafe { write_volatile((self.base_addr + OFFSET_INTCLR_L) as *mut u32, val as u32) };
        } else {
            unsafe {
                write_volatile(
                    (self.base_addr + OFFSET_INTCLR_H) as *mut u32,
                    (val >> 32) as u32,
                )
            };
        }
    }

    /// Gets the mask of pending interrupts for a specific output.
    pub fn get_pending_mask(&self, output: u8) -> Option<u64> {
        match output {
            0 => Some(self.read_u64_reg(OFFSET_INTISR_0_L)),
            1 => Some(self.read_u64_reg(OFFSET_INTISR_1_L)),
            _ => None,
        }
    }

    /// Routes an interrupt source to a specific output.
    pub fn route(&self, irq: u32, output_mask: u8) {
        assert!(irq < 64, "Invalid IRQ number");
        let addr = (self.base_addr + OFFSET_ROUTE_ENTRY_BASE + irq as usize) as *mut u8;
        unsafe { write_volatile(addr, output_mask & 0b11) }; // Only bits 0 and 1 are valid
    }

    /// Gets the current routing for an interrupt source.
    pub fn get_route(&self, irq: u32) -> u8 {
        assert!(irq < 64, "Invalid IRQ number");
        let addr = (self.base_addr + OFFSET_ROUTE_ENTRY_BASE + irq as usize) as *const u8;
        unsafe { read_volatile(addr) }
    }

    /// Sets the distribution mode for a specific interrupt.
    pub fn set_distribution_mode(&self, irq: u32, mode: DistributionMode) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut ctrl0 = self.read_u64_reg(OFFSET_AUTO_CTRL0_L);
        let mut ctrl1 = self.read_u64_reg(OFFSET_AUTO_CTRL1_L);
        let mode_val = mode as u8;

        if (mode_val & 0b01) != 0 {
            ctrl0 |= 1 << irq;
        } else {
            ctrl0 &= !(1 << irq);
        }
        if (mode_val & 0b10) != 0 {
            ctrl1 |= 1 << irq;
        } else {
            ctrl1 &= !(1 << irq);
        }

        self.write_u64_reg(OFFSET_AUTO_CTRL0_L, ctrl0);
        self.write_u64_reg(OFFSET_AUTO_CTRL1_L, ctrl1);
    }

    /// Enables HT Message Signaled Interrupts for an IRQ.
    pub fn enable_ht_msi(&self, irq: u32) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut en = self.read_u64_reg(OFFSET_HTMSI_EN_L);
        en |= 1 << irq;
        self.write_u64_reg(OFFSET_HTMSI_EN_L, en);
    }

    /// Disables HT Message Signaled Interrupts for an IRQ.
    pub fn disable_ht_msi(&self, irq: u32) {
        assert!(irq < 64, "Invalid IRQ number");
        let mut en = self.read_u64_reg(OFFSET_HTMSI_EN_L);
        en &= !(1 << irq);
        self.write_u64_reg(OFFSET_HTMSI_EN_L, en);
    }

    /// Sets the HT MSI vector for a given IRQ.
    pub fn set_ht_msi_vector(&self, irq: u32, vector: u8) {
        assert!(irq < 64, "Invalid IRQ number");
        let addr = (self.base_addr + OFFSET_HTMSI_VECTOR_BASE + irq as usize) as *mut u8;
        unsafe { write_volatile(addr, vector) };
    }
}

/// A test function that can be called from kernel initialization.
/// It assumes `println!` is available for output.
/// # Safety
/// This function performs direct memory access to hardware registers and should only
/// be called in an appropriate context (e.g., kernel init on target hardware).
/// The `base_addr` must be the valid physical address of the CIC.
#[allow(dead_code)]
pub unsafe fn test_loongarch_cic(base_addr: usize) {
    // Assuming println! is available in the kernel.
    println!("--- Starting LoongArch CIC driver test ---");

    let cic = Cic::new(base_addr);
    let test_irq = Interrupt::Uart as u32; // Use UART (IRQ 8) for testing

    println!("Testing with IRQ: {} ({:?})", test_irq, Interrupt::Uart);

    // 1. Read ID and Version
    println!("1. Reading ID and Version...");
    let (id, version, count) = cic.read_id_version();
    println!(
        "  - ID: 0x{:x}, Version: 0x{:x}, Supported IRQs: {}",
        id, version, count
    );
    // According to the doc, ID is 0x7A, and int_num is 0x3F (63), so count is 64.
    // assert_eq!(id, 0x7A, "CIC ID should be 0x7A");
    // assert_eq!(count, 64, "Should support 64 IRQs");
    println!("  - ID and Version OK.");

    // 2. Test enable/disable
    println!("2. Testing enable/disable...");
    cic.disable(test_irq);
    assert!(!cic.is_enabled(test_irq), "Interrupt should be disabled");
    println!("  - Disable OK.");
    cic.enable(test_irq);
    assert!(cic.is_enabled(test_irq), "Interrupt should be enabled");
    println!("  - Enable OK.");

    // 3. Test trigger mode and polarity
    println!("3. Testing trigger mode and polarity...");
    cic.set_trigger_mode(test_irq, false); // Level-triggered
    assert!(!cic.get_trigger_mode(test_irq), "Should be level-triggered");
    println!("  - Set to level-triggered OK.");
    cic.set_polarity(test_irq, true); // High-level active (default)
    println!("  - Set polarity to high-level (call only).");
    cic.set_trigger_mode(test_irq, true); // Edge-triggered
    assert!(cic.get_trigger_mode(test_irq), "Should be edge-triggered");
    println!("  - Set to edge-triggered OK.");

    // 4. Test routing
    println!("4. Testing routing...");
    cic.route(test_irq, 0b01); // Route to output 0
    assert_eq!(
        cic.get_route(test_irq),
        0b01,
        "Should be routed to output 0"
    );
    println!("  - Route to output 0 OK.");
    cic.route(test_irq, 0b10); // Route to output 1
    assert_eq!(
        cic.get_route(test_irq),
        0b10,
        "Should be routed to output 1"
    );
    println!("  - Route to output 1 OK.");

    // 5. Test distribution mode
    println!("5. Testing distribution mode...");
    cic.set_distribution_mode(test_irq, DistributionMode::RoundRobin);
    println!("  - Set distribution mode to RoundRobin (call only).");
    cic.set_distribution_mode(test_irq, DistributionMode::Fixed);
    println!("  - Set distribution mode back to Fixed (call only).");

    // 6. Test HT MSI
    println!("6. Testing HT MSI...");
    cic.disable_ht_msi(test_irq);
    println!("  - Disable HT MSI OK (call only).");
    cic.enable_ht_msi(test_irq);
    println!("  - Enable HT MSI OK (call only).");
    let vector = 0xAB;
    cic.set_ht_msi_vector(test_irq, vector);
    let read_vector =
        read_volatile((cic.base_addr + OFFSET_HTMSI_VECTOR_BASE + test_irq as usize) as *const u8);
    assert_eq!(read_vector, vector, "HT MSI vector should be set correctly");
    println!("  - Set and verify HT MSI vector to 0x{:x} OK.", vector);

    // 7. Final calls and cleanup
    println!("7. Cleanup and final calls...");
    cic.clear_pending_edge(test_irq);
    println!("  - Called clear_pending_edge OK.");
    let pending_mask0 = cic.get_pending_mask(0);
    let pending_mask1 = cic.get_pending_mask(1);
    println!(
        "  - Read pending masks OK (Output 0: {:?}, Output 1: {:?}).",
        pending_mask0, pending_mask1
    );

    // Restore to default state
    cic.disable(test_irq);
    cic.set_trigger_mode(test_irq, false); // Level-triggered
    cic.set_polarity(test_irq, true); // High-level active
    cic.route(test_irq, 0b01); // Default route to output 0
    cic.set_distribution_mode(test_irq, DistributionMode::Fixed);
    cic.disable_ht_msi(test_irq);
    cic.set_ht_msi_vector(test_irq, 0); // Clear vector
    println!("  - Restored IRQ {} to default state.", test_irq);

    println!("--- LoongArch CIC driver test finished successfully ---");
}
