#![no_std]

//! Intel I210/I211 Network Driver for MINIX3
//! 
//! This driver implements support for Intel I210/I211 Gigabit Ethernet controllers
//! Vendor:Device = 8086:1539

use core::panic::PanicInfo;

/// Hardware register definitions for Intel I210/I211
mod registers {
    // Device Control Register
    pub const E1000_CTRL: u32 = 0x00000;
    pub const E1000_CTRL_FD: u32 = 0x00000001;  // Full duplex
    pub const E1000_CTRL_LRST: u32 = 0x00000008; // Link reset
    pub const E1000_CTRL_ASDE: u32 = 0x00000020; // Auto-speed detect
    pub const E1000_CTRL_SLU: u32 = 0x00000040;  // Set link up
    pub const E1000_CTRL_RST: u32 = 0x04000000;  // Device reset
    
    // Device Status Register
    pub const E1000_STATUS: u32 = 0x00008;
    pub const E1000_STATUS_LU: u32 = 0x00000002; // Link up
    pub const E1000_STATUS_SPEED_MASK: u32 = 0x000000C0;
    
    // Interrupt Mask Set/Read Register
    pub const E1000_IMS: u32 = 0x000D0;
    pub const E1000_ICR: u32 = 0x000C0;
    pub const E1000_ICS: u32 = 0x000C8;
    
    // Receive Control Register
    pub const E1000_RCTL: u32 = 0x00100;
    pub const E1000_RCTL_EN: u32 = 0x00000002;   // Receiver Enable
    pub const E1000_RCTL_BAM: u32 = 0x00008000;  // Broadcast Accept Mode
    
    // Transmit Control Register
    pub const E1000_TCTL: u32 = 0x00400;
    pub const E1000_TCTL_EN: u32 = 0x00000002;   // Transmit Enable
    pub const E1000_TCTL_PSP: u32 = 0x00000008;  // Pad Short Packets
    
    // Receive Descriptor Base Address Low
    pub const E1000_RDBAL: u32 = 0x02800;
    pub const E1000_RDBAH: u32 = 0x02804;
    pub const E1000_RDLEN: u32 = 0x02808;
    pub const E1000_RDH: u32 = 0x02810;
    pub const E1000_RDT: u32 = 0x02818;
    
    // Transmit Descriptor Base Address Low
    pub const E1000_TDBAL: u32 = 0x03800;
    pub const E1000_TDBAH: u32 = 0x03804;
    pub const E1000_TDLEN: u32 = 0x03808;
    pub const E1000_TDH: u32 = 0x03810;
    pub const E1000_TDT: u32 = 0x03818;
    
    // EEPROM/Flash Control
    pub const E1000_EECD: u32 = 0x00010;
    
    // MAC Address Registers
    pub const E1000_RAL: u32 = 0x05400;
    pub const E1000_RAH: u32 = 0x05404;
}

/// Driver state structure
#[repr(C)]
pub struct I210State {
    regs: *mut u8,
    irq: i32,
    irq_hook: i32,
    mac_addr: [u8; 6],
}

impl I210State {
    /// Create a new uninitialized driver state
    pub const fn new() -> Self {
        Self {
            regs: core::ptr::null_mut(),
            irq: 0,
            irq_hook: 0,
            mac_addr: [0; 6],
        }
    }
    
    /// Read a 32-bit register
    unsafe fn read_reg(&self, offset: u32) -> u32 {
        if self.regs.is_null() {
            return 0;
        }
        core::ptr::read_volatile(self.regs.add(offset as usize) as *const u32)
    }
    
    /// Write a 32-bit register
    unsafe fn write_reg(&self, offset: u32, value: u32) {
        if self.regs.is_null() {
            return;
        }
        core::ptr::write_volatile(self.regs.add(offset as usize) as *mut u32, value);
    }
}

static mut DRIVER_STATE: I210State = I210State::new();

/// Initialize the driver
/// 
/// # Safety
/// This function must be called only once during driver initialization
#[no_mangle]
pub unsafe extern "C" fn i210_rust_init(regs: *mut u8, irq: i32) -> i32 {
    DRIVER_STATE.regs = regs;
    DRIVER_STATE.irq = irq;
    
    // Perform device reset
    let ctrl = DRIVER_STATE.read_reg(registers::E1000_CTRL);
    DRIVER_STATE.write_reg(registers::E1000_CTRL, ctrl | registers::E1000_CTRL_RST);
    
    // Wait for reset to complete (simplified - real driver would need proper delay)
    for _ in 0..1000 {
        let status = DRIVER_STATE.read_reg(registers::E1000_STATUS);
        if status != 0xFFFFFFFF {
            break;
        }
    }
    
    // Set link up
    let ctrl = DRIVER_STATE.read_reg(registers::E1000_CTRL);
    DRIVER_STATE.write_reg(
        registers::E1000_CTRL,
        ctrl | registers::E1000_CTRL_SLU | registers::E1000_CTRL_ASDE
    );
    
    0 // Success
}

/// Read the MAC address from hardware
/// 
/// # Safety
/// The mac_addr pointer must be valid and point to at least 6 bytes
#[no_mangle]
pub unsafe extern "C" fn i210_rust_get_mac_addr(mac_addr: *mut u8) -> i32 {
    if mac_addr.is_null() {
        return -1;
    }
    
    // Read MAC address from hardware registers
    let ral = DRIVER_STATE.read_reg(registers::E1000_RAL);
    let rah = DRIVER_STATE.read_reg(registers::E1000_RAH);
    
    *mac_addr.add(0) = (ral & 0xFF) as u8;
    *mac_addr.add(1) = ((ral >> 8) & 0xFF) as u8;
    *mac_addr.add(2) = ((ral >> 16) & 0xFF) as u8;
    *mac_addr.add(3) = ((ral >> 24) & 0xFF) as u8;
    *mac_addr.add(4) = (rah & 0xFF) as u8;
    *mac_addr.add(5) = ((rah >> 8) & 0xFF) as u8;
    
    // Store in driver state
    for i in 0..6 {
        DRIVER_STATE.mac_addr[i] = *mac_addr.add(i);
    }
    
    0 // Success
}

/// Enable receiver
#[no_mangle]
pub unsafe extern "C" fn i210_rust_enable_rx() -> i32 {
    let rctl = DRIVER_STATE.read_reg(registers::E1000_RCTL);
    DRIVER_STATE.write_reg(
        registers::E1000_RCTL,
        rctl | registers::E1000_RCTL_EN | registers::E1000_RCTL_BAM
    );
    0
}

/// Enable transmitter
#[no_mangle]
pub unsafe extern "C" fn i210_rust_enable_tx() -> i32 {
    let tctl = DRIVER_STATE.read_reg(registers::E1000_TCTL);
    DRIVER_STATE.write_reg(
        registers::E1000_TCTL,
        tctl | registers::E1000_TCTL_EN | registers::E1000_TCTL_PSP
    );
    0
}

/// Get link status
#[no_mangle]
pub unsafe extern "C" fn i210_rust_get_link_status() -> i32 {
    let status = DRIVER_STATE.read_reg(registers::E1000_STATUS);
    if status & registers::E1000_STATUS_LU != 0 {
        1 // Link is up
    } else {
        0 // Link is down
    }
}

/// Reset the device
#[no_mangle]
pub unsafe extern "C" fn i210_rust_reset() -> i32 {
    let ctrl = DRIVER_STATE.read_reg(registers::E1000_CTRL);
    DRIVER_STATE.write_reg(registers::E1000_CTRL, ctrl | registers::E1000_CTRL_RST);
    0
}

/// Panic handler for no_std environment
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
