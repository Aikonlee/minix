// I210/I211 Ethernet Controller Driver Core Library
// This library provides low-level hardware access and management for Intel I210/I211 NICs

// Temporarily use std for simplicity
// #![no_std]

// extern crate alloc;
// use alloc::boxed::Box;

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
pub mod registers {
    pub const CTRL: u32 = 0x0000;        // Device Control Register
    pub const STATUS: u32 = 0x0008;      // Device Status Register
    pub const EECD: u32 = 0x0010;        // EEPROM/Flash Control & Data Register
    pub const EERD: u32 = 0x0014;        // EEPROM Read Register
    pub const CTRL_EXT: u32 = 0x0018;    // Extended Device Control Register
    pub const FLA: u32 = 0x001C;         // Flash Access Register
    pub const MDIC: u32 = 0x0020;        // MDI Control Register
    pub const SCTL: u32 = 0x0024;        // SerDes Control Register
    pub const EXPANSION_ROM_BASE: u32 = 0x0028; // Expansion ROM Base Address Register
    pub const LEDCTL: u32 = 0x00E0;      // LED Control Register
    pub const PBA: u32 = 0x1000;         // Packet Buffer Allocation Register
    pub const PBS: u32 = 0x1008;         // Packet Buffer Size Register
    pub const EEC: u32 = 0x100C;         // EEPROM Control Register
    pub const FLA2: u32 = 0x1018;        // Flash Access Register 2
    pub const RSRPD: u32 = 0x2C00;       // Receive Small Packet Detect
    pub const RDFH: u32 = 0x2410;        // Receive Data FIFO Head
    pub const RDFT: u32 = 0x2420;        // Receive Data FIFO Tail
    pub const RDFHS: u32 = 0x2424;       // Receive Data FIFO Head Saved
    pub const RDFTS: u32 = 0x2428;       // Receive Data FIFO Tail Saved
    pub const RDFPC: u32 = 0x24F0;       // Receive Data FIFO Packet Count
    pub const RDBAL: u32 = 0x2800;       // Receive Descriptor Base Address Low
    pub const RDBAH: u32 = 0x2804;       // Receive Descriptor Base Address High
    pub const RDLEN: u32 = 0x2808;       // Receive Descriptor Length
    pub const RDH: u32 = 0x2810;         // Receive Descriptor Head
    pub const RDT: u32 = 0x2818;         // Receive Descriptor Tail
    pub const RDTR: u32 = 0x2820;        // Receive Delay Timer Register
    pub const RXDCTL: u32 = 0x2828;      // Receive Descriptor Control
    pub const RADV: u32 = 0x282C;        // Receive Interrupt Absolute Delay Timer
    pub const RA: u32 = 0x5400;          // Receive Address Register
    pub const MTA: u32 = 0x5200;         // Multicast Table Array
    pub const TDBAL: u32 = 0x3800;       // Transmit Descriptor Base Address Low
    pub const TDBAH: u32 = 0x3804;       // Transmit Descriptor Base Address High
    pub const TDLEN: u32 = 0x3808;       // Transmit Descriptor Length
    pub const TDH: u32 = 0x3810;         // Transmit Descriptor Head
    pub const TDT: u32 = 0x3818;         // Transmit Descriptor Tail
    pub const TIDV: u32 = 0x3820;        // Transmit Interrupt Delay Value
    pub const TXDCTL: u32 = 0x3828;      // Transmit Descriptor Control
    pub const TADV: u32 = 0x382C;        // Transmit Absolute Interrupt Delay Value
    pub const TSPMT: u32 = 0x3830;       // TCP Segmentation Pad & Min Threshold
    pub const CRCERRS: u32 = 0x4000;     // CRC Error Count
    pub const MPC: u32 = 0x4010;         // Missed Packets Count
    pub const SCC: u32 = 0x4014;         // Single Collision Count
    pub const ECOL: u32 = 0x4018;        // Excessive Collision Count
    pub const MCC: u32 = 0x401C;         // Multiple Collision Count
    pub const LATECOL: u32 = 0x4020;     // Late Collision Count
    pub const COLC: u32 = 0x4028;        // Collision Count
    pub const DC: u32 = 0x4030;          // Defer Count
    pub const TNCRS: u32 = 0x4040;       // Transmit with No CRS
    pub const SEC: u32 = 0x4044;         // Sequence Error Count
    pub const CEXTERR: u32 = 0x4048;     // Carrier Extension Error Count
    pub const RLEC: u32 = 0x404C;        // Receive Length Error Count
    pub const XONRXC: u32 = 0x4050;      // XON Received Count
    pub const XONTXC: u32 = 0x4054;      // XON Transmitted Count
    pub const XOFFRXC: u32 = 0x4058;     // XOFF Received Count
    pub const XOFFTXC: u32 = 0x405C;     // XOFF Transmitted Count
    pub const FCRUC: u32 = 0x4060;       // Flow Control Received Unsupported Count
    pub const PRC64: u32 = 0x4064;       // Packets Received (64 Bytes)
    pub const PRC127: u32 = 0x4068;      // Packets Received (65-127 Bytes)
    pub const PRC255: u32 = 0x406C;      // Packets Received (128-255 Bytes)
    pub const PRC511: u32 = 0x4070;      // Packets Received (256-511 Bytes)
    pub const PRC1023: u32 = 0x4074;     // Packets Received (512-1023 Bytes)
    pub const PRC1522: u32 = 0x4078;     // Packets Received (1024-1522 Bytes)
    pub const GPRC: u32 = 0x407C;        // Good Packets Received Count
    pub const BPRC: u32 = 0x4080;        // Broadcast Packets Received Count
    pub const MPRC: u32 = 0x4084;        // Multicast Packets Received Count
    pub const GPTC: u32 = 0x4088;        // Good Packets Transmitted Count
    pub const GORCL: u32 = 0x408C;       // Good Octets Received Count Low
    pub const GORCH: u32 = 0x4090;       // Good Octets Received Count High
    pub const GOTCL: u32 = 0x4094;       // Good Octets Transmitted Count Low
    pub const GOTCH: u32 = 0x4098;       // Good Octets Transmitted Count High
    pub const RNBC: u32 = 0x40A0;        // Receive No Buffers Count
    pub const RUC: u32 = 0x40A4;         // Receive Undersize Count
    pub const RFC: u32 = 0x40A8;         // Receive Fragment Count
    pub const ROC: u32 = 0x40AC;         // Receive Oversize Count
    pub const RJC: u32 = 0x40B0;         // Receive Jabber Count
    pub const MGTPRC: u32 = 0x40B4;      // Management Packets Received Count
    pub const MGTPDC: u32 = 0x40B8;      // Management Packets Dropped Count
    pub const MGTPTC: u32 = 0x40BC;      // Management Packets Transmitted Count
    pub const TORL: u32 = 0x40C0;        // Total Octets Received
    pub const TORH: u32 = 0x40C4;        // Total Octets Received
    pub const TOTL: u32 = 0x40C8;        // Total Octets Transmitted
    pub const TOTH: u32 = 0x40CC;        // Total Octets Transmitted
    pub const TPR: u32 = 0x40D0;         // Total Packets Received
    pub const TPT: u32 = 0x40D4;         // Total Packets Transmitted
    pub const PTC64: u32 = 0x40D8;       // Packets Transmitted (64 Bytes)
    pub const PTC127: u32 = 0x40DC;      // Packets Transmitted (65-127 Bytes)
    pub const PTC255: u32 = 0x40E0;      // Packets Transmitted (128-255 Bytes)
    pub const PTC511: u32 = 0x40E4;      // Packets Transmitted (256-511 Bytes)
    pub const PTC1023: u32 = 0x40E8;     // Packets Transmitted (512-1023 Bytes)
    pub const PTC1522: u32 = 0x40EC;     // Packets Transmitted (1024-1522 Bytes)
    pub const MPTC: u32 = 0x40F0;        // Multicast Packets Transmitted Count
    pub const BPTC: u32 = 0x40F4;        // Broadcast Packets Transmitted Count
    pub const TSCTC: u32 = 0x40F8;       // TCP Segmentation Context Transmitted Count
    pub const TSCTFC: u32 = 0x40FC;      // TCP Segmentation Context Transmit Fail Count
    pub const IAC: u32 = 0x4100;         // Interrupt Assertion Count
    pub const ICR: u32 = 0x00C0;         // Interrupt Cause Read Register
    pub const ITR: u32 = 0x00C4;         // Interrupt Throttling Register
    pub const IMS: u32 = 0x00D0;         // Interrupt Mask Set/Read Register
    pub const IMC: u32 = 0x00D8;         // Interrupt Mask Clear Register
    pub const IAM: u32 = 0x00E0;         // Interrupt Acknowledge Auto Mask Register
}

// Driver state management
#[derive(Debug)]
pub struct I210Driver {
    pub base_addr: *mut u32,
    pub mac_addr: [u8; 6],
    pub link_up: bool,
}

impl I210Driver {
    pub fn new(base_addr: *mut u32) -> Self {
        I210Driver {
            base_addr,
            mac_addr: [0; 6],
            link_up: false,
        }
    }

    // Hardware initialization and reset
    pub fn init(&mut self) {
        // Reset the device
        self.reset();

        // Read MAC address
        self.read_mac_address();

        // Initialize receive and transmit
        self.init_receive();
        self.init_transmit();

        // Enable interrupts
        self.enable_interrupts();
    }

    pub fn reset(&self) {
        unsafe {
            // Set reset bit in CTRL register
            let ctrl = self.read_reg(registers::CTRL);
            self.write_reg(registers::CTRL, ctrl | (1 << 26)); // RST bit

            // Wait for reset to complete
            while (self.read_reg(registers::CTRL) & (1 << 26)) != 0 {}
        }
    }

    // MAC address reading
    pub fn read_mac_address(&mut self) {
        unsafe {
            // Read MAC address from EEPROM or registers
            // For I210, MAC is stored in RA register
            let ral = self.read_reg(registers::RA);
            let rah = self.read_reg(registers::RA + 4);

            self.mac_addr[0] = (ral & 0xFF) as u8;
            self.mac_addr[1] = ((ral >> 8) & 0xFF) as u8;
            self.mac_addr[2] = ((ral >> 16) & 0xFF) as u8;
            self.mac_addr[3] = ((ral >> 24) & 0xFF) as u8;
            self.mac_addr[4] = (rah & 0xFF) as u8;
            self.mac_addr[5] = ((rah >> 8) & 0xFF) as u8;
        }
    }

    // RX/TX enable functions
    pub fn init_receive(&self) {
        unsafe {
            // Set receive descriptor base address (dummy for now)
            self.write_reg(registers::RDBAL, 0);
            self.write_reg(registers::RDBAH, 0);

            // Set receive descriptor length
            self.write_reg(registers::RDLEN, 128); // 8 descriptors * 16 bytes

            // Set receive descriptor head and tail
            self.write_reg(registers::RDH, 0);
            self.write_reg(registers::RDT, 7); // 8 descriptors - 1

            // Enable receive
            let rxdctl = self.read_reg(registers::RXDCTL);
            self.write_reg(registers::RXDCTL, rxdctl | (1 << 25)); // RX enable
        }
    }

    pub fn init_transmit(&self) {
        unsafe {
            // Set transmit descriptor base address (dummy for now)
            self.write_reg(registers::TDBAL, 0);
            self.write_reg(registers::TDBAH, 0);

            // Set transmit descriptor length
            self.write_reg(registers::TDLEN, 128); // 8 descriptors * 16 bytes

            // Set transmit descriptor head and tail
            self.write_reg(registers::TDH, 0);
            self.write_reg(registers::TDT, 0);

            // Enable transmit
            let txdctl = self.read_reg(registers::TXDCTL);
            self.write_reg(registers::TXDCTL, txdctl | (1 << 25)); // TX enable
        }
    }

    // Link status detection
    pub fn check_link_status(&mut self) {
        unsafe {
            let status = self.read_reg(registers::STATUS);
            self.link_up = (status & (1 << 1)) != 0; // LU bit
        }
    }

    pub fn enable_interrupts(&self) {
        unsafe {
            // Enable basic interrupts
            self.write_reg(registers::IMS, (1 << 0) | (1 << 1) | (1 << 2)); // TXDW, TXQE, LSC
        }
    }

    // Low-level register access
    pub unsafe fn read_reg(&self, offset: u32) -> u32 {
        core::ptr::read_volatile(self.base_addr.add((offset / 4) as usize))
    }

    pub unsafe fn write_reg(&self, offset: u32, value: u32) {
        core::ptr::write_volatile(self.base_addr.add((offset / 4) as usize), value);
    }
}

// FFI interface for C integration
#[no_mangle]
pub extern "C" fn i210_init(base_addr: *mut u32) -> *mut I210Driver {
    let driver = Box::new(I210Driver::new(base_addr));
    Box::into_raw(driver)
}

#[no_mangle]
pub extern "C" fn i210_reset(driver: *mut I210Driver) {
    unsafe {
        (*driver).reset();
    }
}

#[no_mangle]
pub extern "C" fn i210_read_mac(driver: *mut I210Driver, mac: *mut u8) {
    unsafe {
        (*driver).read_mac_address();
        for i in 0..6 {
            *mac.add(i) = (*driver).mac_addr[i];
        }
    }
}

#[no_mangle]
pub extern "C" fn i210_check_link(driver: *mut I210Driver) -> bool {
    unsafe {
        (*driver).check_link_status();
        (*driver).link_up
    }
}

#[no_mangle]
pub extern "C" fn i210_enable_rx(driver: *mut I210Driver) {
    unsafe {
        (*driver).init_receive();
    }
}

#[no_mangle]
pub extern "C" fn i210_enable_tx(driver: *mut I210Driver) {
    unsafe {
        (*driver).init_transmit();
    }
}