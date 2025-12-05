#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(not(feature = "std"))]
use core::hint::spin_loop;
#[cfg(not(feature = "std"))]
use core::panic::PanicInfo;

#[cfg(not(feature = "std"))]
extern "C" {
	fn malloc(size: usize) -> *mut c_void;
	fn free(ptr: *mut c_void);
}

pub mod registers {
	pub const CTRL: u32 = 0x0000;
	pub const STATUS: u32 = 0x0008;
	pub const EERD: u32 = 0x0014;
	pub const RCTL: u32 = 0x0100;
	pub const TCTL: u32 = 0x0400;
	pub const TIPG: u32 = 0x0410;
	pub const ICR: u32 = 0x00C0;
	pub const IMS: u32 = 0x00D0;
	pub const IMC: u32 = 0x00D8;
	pub const RDBAL: u32 = 0x2800;
	pub const RDBAH: u32 = 0x2804;
	pub const RDLEN: u32 = 0x2808;
	pub const RDH: u32 = 0x2810;
	pub const RDT: u32 = 0x2818;
	pub const RDTR: u32 = 0x2820;
	pub const RXDCTL: u32 = 0x2828;
	pub const RADV: u32 = 0x282C;
	pub const TDBAL: u32 = 0x3800;
	pub const TDBAH: u32 = 0x3804;
	pub const TDLEN: u32 = 0x3808;
	pub const TDH: u32 = 0x3810;
	pub const TDT: u32 = 0x3818;
	pub const TIDV: u32 = 0x3820;
	pub const TXDCTL: u32 = 0x3828;
	pub const TADV: u32 = 0x382C;
	pub const RA: u32 = 0x5400;
	pub const MTA: u32 = 0x5200;
}

const OK: i32 = 0;
const EINVAL: i32 = 22;
const ENOMEM: i32 = 12;
const EAGAIN: i32 = 11;
const EBUSY: i32 = 16;

const I210_RX_BUFFER_SIZE: usize = 2048;
const I210_TX_BUFFER_SIZE: usize = 2048;

const CTRL_RST: u32 = 1 << 26;
const RCTL_EN: u32 = 1 << 1;
const RCTL_SBP: u32 = 1 << 2;
const RCTL_UPE: u32 = 1 << 3;
const RCTL_MPE: u32 = 1 << 4;
const RCTL_BAM: u32 = 1 << 15;
const RCTL_BSIZE_MASK: u32 = (1 << 16) | (1 << 17);
const RCTL_SECRC: u32 = 1 << 26;

const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;

const TCTL_CT_SHIFT: u32 = 4;
const TCTL_COLD_SHIFT: u32 = 12;

const IMS_TXDW: u32 = 1 << 0;
const IMS_TXQE: u32 = 1 << 1;
const IMS_LSC: u32 = 1 << 2;
const IMS_RXO: u32 = 1 << 6;
const IMS_RXT: u32 = 1 << 7;

const I210_EVENT_RX: u32 = 0x1;
const I210_EVENT_TX: u32 = 0x2;
const I210_EVENT_LINK: u32 = 0x4;

const NDEV_MODE_BCAST: u32 = 0x02;
const NDEV_MODE_MCAST_LIST: u32 = 0x04;
const NDEV_MODE_MCAST_ALL: u32 = 0x08;
const NDEV_MODE_PROMISC: u32 = 0x10;

#[repr(C)]
pub struct DmaBlock {
	pub virt: *mut c_void,
	pub phys: u64,
	pub size: usize,
}

#[repr(C)]
pub struct BufferView {
	pub ptr: *mut u8,
	pub len: usize,
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
struct RxDesc {
	buffer: u32,
	buffer_h: u32,
	length: u16,
	checksum: u16,
	status: u8,
	errors: u8,
	special: u16,
}

impl Default for RxDesc {
	fn default() -> Self {
		Self {
			buffer: 0,
			buffer_h: 0,
			length: 0,
			checksum: 0,
			status: 0,
			errors: 0,
			special: 0,
		}
	}
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
struct TxDesc {
	buffer: u32,
	buffer_h: u32,
	length: u16,
	cso: u8,
	cmd: u8,
	status: u8,
	css: u8,
	special: u16,
}

impl Default for TxDesc {
	fn default() -> Self {
		Self {
			buffer: 0,
			buffer_h: 0,
			length: 0,
			cso: 0,
			cmd: 0,
			status: 0,
			css: 0,
			special: 0,
		}
	}
}

const RX_STATUS_DD: u8 = 0x01;
const RX_STATUS_EOP: u8 = 0x02;
const TX_CMD_EOP: u8 = 0x01;
const TX_CMD_RS: u8 = 0x08;
const TX_CMD_IFCS: u8 = 0x02;

pub struct I210Driver {
	regs: *mut u8,
	rx_desc: *mut RxDesc,
	tx_desc: *mut TxDesc,
	rx_desc_phys: u64,
	tx_desc_phys: u64,
	rx_buf: *mut u8,
	tx_buf: *mut u8,
	rx_buf_phys: u64,
	tx_buf_phys: u64,
	rx_count: usize,
	tx_count: usize,
	rx_tail: usize,
	tx_tail: usize,
	rx_next: usize,
	rx_pending: Option<usize>,
	tx_pending: Option<usize>,
	mac_addr: [u8; 6],
}

impl I210Driver {
	fn new(regs: *mut u8) -> Self {
		Self {
			regs,
			rx_desc: ptr::null_mut(),
			tx_desc: ptr::null_mut(),
			rx_desc_phys: 0,
			tx_desc_phys: 0,
			rx_buf: ptr::null_mut(),
			tx_buf: ptr::null_mut(),
			rx_buf_phys: 0,
			tx_buf_phys: 0,
			rx_count: 0,
			tx_count: 0,
			rx_tail: 0,
			tx_tail: 0,
			rx_next: 0,
			rx_pending: None,
			tx_pending: None,
			mac_addr: [0; 6],
		}
	}

	unsafe fn read_reg(&self, offset: u32) -> u32 {
		ptr::read_volatile(self.regs.add(offset as usize) as *const u32)
	}

	unsafe fn write_reg(&self, offset: u32, value: u32) {
		ptr::write_volatile(self.regs.add(offset as usize) as *mut u32, value);
	}

	unsafe fn reg_pair_write(&self, low: u32, high: u32, value: u64) {
		self.write_reg(low, value as u32);
		self.write_reg(high, (value >> 32) as u32);
	}

	unsafe fn reset(&mut self) {
		let ctrl = self.read_reg(registers::CTRL);
		self.write_reg(registers::CTRL, ctrl | CTRL_RST);
		while (self.read_reg(registers::CTRL) & CTRL_RST) != 0 {}
	}

	unsafe fn setup_dma(&mut self, rx_desc: &DmaBlock, rx_buf: &DmaBlock,
		tx_desc: &DmaBlock, tx_buf: &DmaBlock,
		rx_count: usize, tx_count: usize) -> i32
	{
		if rx_desc.virt.is_null() || rx_buf.virt.is_null() ||
			tx_desc.virt.is_null() || tx_buf.virt.is_null() ||
			rx_count == 0 || tx_count == 0
		{
			return EINVAL;
		}

		let rx_bytes = rx_count * size_of::<RxDesc>();
		let tx_bytes = tx_count * size_of::<TxDesc>();
		if rx_desc.size < rx_bytes || tx_desc.size < tx_bytes {
			return EINVAL;
		}

		let rx_buf_bytes = rx_count * I210_RX_BUFFER_SIZE;
		let tx_buf_bytes = tx_count * I210_TX_BUFFER_SIZE;
		if rx_buf.size < rx_buf_bytes || tx_buf.size < tx_buf_bytes {
			return EINVAL;
		}

		self.rx_desc = rx_desc.virt as *mut RxDesc;
		self.tx_desc = tx_desc.virt as *mut TxDesc;
		self.rx_desc_phys = rx_desc.phys;
		self.tx_desc_phys = tx_desc.phys;
		self.rx_buf = rx_buf.virt as *mut u8;
		self.tx_buf = tx_buf.virt as *mut u8;
		self.rx_buf_phys = rx_buf.phys;
		self.tx_buf_phys = tx_buf.phys;
		self.rx_count = rx_count;
		self.tx_count = tx_count;
		self.rx_tail = rx_count.saturating_sub(1);
		self.tx_tail = 0;
		self.rx_next = 0;
		self.rx_pending = None;
		self.tx_pending = None;

		ptr::write_bytes(self.rx_desc, 0, rx_count);
		ptr::write_bytes(self.tx_desc, 0, tx_count);

		for i in 0..rx_count {
			let desc = &mut *self.rx_desc.add(i);
			let addr = self.rx_buf_phys + (i * I210_RX_BUFFER_SIZE) as u64;
			desc.buffer = addr as u32;
			desc.buffer_h = (addr >> 32) as u32;
			desc.status = 0;
		}

		for i in 0..tx_count {
			let desc = &mut *self.tx_desc.add(i);
			let addr = self.tx_buf_phys + (i * I210_TX_BUFFER_SIZE) as u64;
			desc.buffer = addr as u32;
			desc.buffer_h = (addr >> 32) as u32;
			desc.status = 0;
		}

		self.reg_pair_write(registers::RDBAL, registers::RDBAH, self.rx_desc_phys);
		self.write_reg(registers::RDLEN, rx_bytes as u32);
		self.write_reg(registers::RDH, 0);
		self.write_reg(registers::RDT, self.rx_tail as u32);

		self.reg_pair_write(registers::TDBAL, registers::TDBAH, self.tx_desc_phys);
		self.write_reg(registers::TDLEN, tx_bytes as u32);
		self.write_reg(registers::TDH, 0);
		self.write_reg(registers::TDT, 0);

		OK
	}

	unsafe fn configure_rctl(&mut self) {
		let mut rctl = self.read_reg(registers::RCTL);
		rctl &= !(RCTL_BSIZE_MASK | RCTL_SBP);
		rctl |= RCTL_EN | RCTL_SECRC | RCTL_BAM;
		self.write_reg(registers::RCTL, rctl);
	}

	unsafe fn configure_tctl(&mut self) {
		let mut tctl = self.read_reg(registers::TCTL);
		tctl |= TCTL_EN | TCTL_PSP;
		tctl &= !(((0xFFFF) << TCTL_CT_SHIFT) | ((0x3FF) << TCTL_COLD_SHIFT));
		tctl |= (0x10 << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT);
		self.write_reg(registers::TCTL, tctl);
		self.write_reg(registers::TIPG, 0x0060200A);
	}

	unsafe fn start(&mut self) {
		self.configure_rctl();
		self.configure_tctl();
		self.write_reg(registers::RXDCTL,
			self.read_reg(registers::RXDCTL) | (1 << 25));
		self.write_reg(registers::TXDCTL,
			self.read_reg(registers::TXDCTL) | (1 << 25));
		self.rx_tail = self.rx_count.saturating_sub(1);
		self.rx_next = 0;
		self.write_reg(registers::RDT, self.rx_tail as u32);
		self.set_rx_mode(NDEV_MODE_BCAST | NDEV_MODE_MCAST_LIST);
	}

	unsafe fn stop(&mut self) {
		let mut rctl = self.read_reg(registers::RCTL);
		rctl &= !RCTL_EN;
		self.write_reg(registers::RCTL, rctl);

		let mut tctl = self.read_reg(registers::TCTL);
		tctl &= !TCTL_EN;
		self.write_reg(registers::TCTL, tctl);
	}

	unsafe fn read_mac(&mut self) {
		let ral = self.read_reg(registers::RA);
		let rah = self.read_reg(registers::RA + 4);
		self.mac_addr[0] = (ral & 0xFF) as u8;
		self.mac_addr[1] = ((ral >> 8) & 0xFF) as u8;
		self.mac_addr[2] = ((ral >> 16) & 0xFF) as u8;
		self.mac_addr[3] = ((ral >> 24) & 0xFF) as u8;
		self.mac_addr[4] = (rah & 0xFF) as u8;
		self.mac_addr[5] = ((rah >> 8) & 0xFF) as u8;
	}

	unsafe fn set_mac(&mut self, mac: &[u8; 6]) {
		let ral: u32 = (mac[3] as u32) << 24
			| (mac[2] as u32) << 16
			| (mac[1] as u32) << 8
			| mac[0] as u32;
		let mut rah: u32 = (mac[5] as u32) << 8 | mac[4] as u32;
		rah |= 1 << 31;
		self.write_reg(registers::RA, ral);
		self.write_reg(registers::RA + 4, rah);
		self.mac_addr = *mac;
	}

	unsafe fn enable_interrupts(&self) {
		self.write_reg(registers::IMS, IMS_TXDW | IMS_TXQE | IMS_LSC | IMS_RXO | IMS_RXT);
	}

	unsafe fn disable_interrupts(&self) {
		self.write_reg(registers::IMC, 0xFFFF_FFFF);
	}

	unsafe fn set_rx_mode(&mut self, mode: u32) {
		let mut rctl = self.read_reg(registers::RCTL);
		rctl &= !(RCTL_BAM | RCTL_MPE | RCTL_UPE);
		if (mode & NDEV_MODE_BCAST) != 0 {
			rctl |= RCTL_BAM;
		}
		if (mode & (NDEV_MODE_MCAST_LIST | NDEV_MODE_MCAST_ALL)) != 0 {
			rctl |= RCTL_MPE;
		}
		if (mode & NDEV_MODE_PROMISC) != 0 {
			rctl |= RCTL_BAM | RCTL_MPE | RCTL_UPE;
		}
		self.write_reg(registers::RCTL, rctl);
	}

	unsafe fn get_tx_buffer(&mut self, view: *mut BufferView) -> i32 {
		if self.tx_pending.is_some() {
			return EBUSY;
		}
		let head = (self.read_reg(registers::TDH) as usize) % self.tx_count.max(1);
		let next = (self.tx_tail + 1) % self.tx_count;
		if next == head {
			return EAGAIN;
		}
		let ptr = self.tx_buf.add(self.tx_tail * I210_TX_BUFFER_SIZE);
		if view.is_null() {
			return EINVAL;
		}
		(*view).ptr = ptr;
		(*view).len = I210_TX_BUFFER_SIZE;
		self.tx_pending = Some(self.tx_tail);
		OK
	}

	unsafe fn commit_tx(&mut self, len: usize) -> i32 {
		let index = match self.tx_pending.take() {
			Some(i) => i,
			None => return EINVAL,
		};
		if len == 0 || len > I210_TX_BUFFER_SIZE {
			return EINVAL;
		}
		let desc = &mut *self.tx_desc.add(index);
		desc.length = len as u16;
		desc.status = 0;
		desc.cmd = TX_CMD_EOP | TX_CMD_RS | TX_CMD_IFCS;
		desc.cso = 0;
		desc.css = 0;
		compiler_fence(Ordering::SeqCst);
		self.tx_tail = (index + 1) % self.tx_count;
		self.write_reg(registers::TDT, self.tx_tail as u32);
		OK
	}

	unsafe fn peek_rx(&mut self, view: *mut BufferView) -> i32 {
		if view.is_null() {
			return EINVAL;
		}
		if self.rx_pending.is_some() {
			return EBUSY;
		}
		let index = self.rx_next;
		let desc = &mut *self.rx_desc.add(index);
		if (desc.status & RX_STATUS_DD) == 0 {
			return EAGAIN;
		}
		if (desc.status & RX_STATUS_EOP) == 0 {
			desc.status = 0;
			self.write_reg(registers::RDT, index as u32);
			self.rx_next = (index + 1) % self.rx_count;
			return EAGAIN;
		}
		let ptr = self.rx_buf.add(index * I210_RX_BUFFER_SIZE);
		(*view).ptr = ptr;
		(*view).len = desc.length as usize;
		self.rx_pending = Some(index);
		OK
	}

	unsafe fn release_rx(&mut self) {
		if let Some(index) = self.rx_pending.take() {
			let desc = &mut *self.rx_desc.add(index);
			desc.status = 0;
			compiler_fence(Ordering::SeqCst);
			self.write_reg(registers::RDT, index as u32);
			self.rx_next = (index + 1) % self.rx_count;
			self.rx_tail = index;
		}
	}

	unsafe fn handle_interrupts(&self) -> u32 {
		let cause = self.read_reg(registers::ICR);
		if cause == 0 {
			return 0;
		}
		let mut events = 0;
		if (cause & (IMS_RXO | IMS_RXT)) != 0 {
			events |= I210_EVENT_RX;
		}
		if (cause & (IMS_TXDW | IMS_TXQE)) != 0 {
			events |= I210_EVENT_TX;
		}
		if (cause & IMS_LSC) != 0 {
			events |= I210_EVENT_LINK;
		}
		events
	}

	unsafe fn link_up(&self) -> bool {
		(self.read_reg(registers::STATUS) & (1 << 1)) != 0
	}
}

#[cfg(feature = "std")]
fn driver_alloc(driver: I210Driver) -> *mut I210Driver {
	Box::into_raw(Box::new(driver))
}

#[cfg(feature = "std")]
unsafe fn driver_free(driver: *mut I210Driver) {
	if !driver.is_null() {
		drop(Box::from_raw(driver));
	}
}

#[cfg(not(feature = "std"))]
fn driver_alloc(driver: I210Driver) -> *mut I210Driver {
	unsafe {
		let mem = malloc(size_of::<I210Driver>()) as *mut I210Driver;
		if mem.is_null() {
			ptr::null_mut()
		} else {
			ptr::write(mem, driver);
			mem
		}
	}
}

#[cfg(not(feature = "std"))]
unsafe fn driver_free(driver: *mut I210Driver) {
	if driver.is_null() {
		return;
	}
	ptr::drop_in_place(driver);
	free(driver as *mut c_void);
}

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
	loop {
		spin_loop();
	}
}

fn driver_from_ptr(driver: *mut I210Driver) -> Option<&'static mut I210Driver> {
	unsafe { driver.as_mut() }
}

#[no_mangle]
pub extern "C" fn i210_hw_create(base: *mut c_void) -> *mut I210Driver {
	if base.is_null() {
		return ptr::null_mut();
	}
	let driver = I210Driver::new(base as *mut u8);
	let ptr = driver_alloc(driver);
	if ptr.is_null() {
		return ptr;
	}
	ptr
}

#[no_mangle]
pub extern "C" fn i210_hw_destroy(driver: *mut I210Driver) {
	if driver.is_null() {
		return;
	}
	unsafe {
		driver_free(driver);
	}
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn bcmp(lhs: *const u8, rhs: *const u8, len: usize) -> i32 {
	if len == 0 {
		return 0;
	}
	unsafe {
		for i in 0..len {
			if *lhs.add(i) != *rhs.add(i) {
				return 1;
			}
		}
	}
	0
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[no_mangle]
pub extern "C" fn i210_hw_setup_dma(driver: *mut I210Driver,
	rx_desc: *const DmaBlock, rx_buf: *const DmaBlock,
	tx_desc: *const DmaBlock, tx_buf: *const DmaBlock,
	rx_count: usize, tx_count: usize) -> i32
{
	let drv = match driver_from_ptr(driver) {
		Some(d) => d,
		None => return EINVAL,
	};
	if rx_desc.is_null() || rx_buf.is_null() || tx_desc.is_null() || tx_buf.is_null() {
		return EINVAL;
	}
	unsafe {
		drv.setup_dma(&*rx_desc, &*rx_buf, &*tx_desc, &*tx_buf, rx_count, tx_count)
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_start(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.start();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_stop(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.stop();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_reset(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.reset();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_read_mac(driver: *mut I210Driver, mac: *mut u8) {
	if mac.is_null() {
		return;
	}
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.read_mac();
			for i in 0..6 {
				*mac.add(i) = drv.mac_addr[i];
			}
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_set_mac(driver: *mut I210Driver, mac: *const u8) -> i32 {
	if mac.is_null() {
		return EINVAL;
	}
	let drv = match driver_from_ptr(driver) {
		Some(d) => d,
		None => return EINVAL,
	};
	let mut addr = [0u8; 6];
	unsafe {
		ptr::copy_nonoverlapping(mac, addr.as_mut_ptr(), 6);
		drv.set_mac(&addr);
	}
	OK
}

#[no_mangle]
pub extern "C" fn i210_hw_link_status(driver: *mut I210Driver) -> i32 {
	match driver_from_ptr(driver) {
		Some(drv) => unsafe { drv.link_up() as i32 },
		None => 0,
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_get_tx_buffer(driver: *mut I210Driver,
	view: *mut BufferView) -> i32
{
	let drv = match driver_from_ptr(driver) {
		Some(d) => d,
		None => return EINVAL,
	};
	unsafe { drv.get_tx_buffer(view) }
}

#[no_mangle]
pub extern "C" fn i210_hw_commit_tx(driver: *mut I210Driver, len: usize) -> i32 {
	let drv = match driver_from_ptr(driver) {
		Some(d) => d,
		None => return EINVAL,
	};
	unsafe { drv.commit_tx(len) }
}

#[no_mangle]
pub extern "C" fn i210_hw_peek_rx(driver: *mut I210Driver,
	view: *mut BufferView) -> i32
{
	let drv = match driver_from_ptr(driver) {
		Some(d) => d,
		None => return EINVAL,
	};
	unsafe { drv.peek_rx(view) }
}

#[no_mangle]
pub extern "C" fn i210_hw_release_rx(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.release_rx();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_handle_interrupts(driver: *mut I210Driver) -> u32 {
	match driver_from_ptr(driver) {
		Some(drv) => unsafe { drv.handle_interrupts() },
		None => 0,
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_enable_interrupts(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.enable_interrupts();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_disable_interrupts(driver: *mut I210Driver) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.disable_interrupts();
		}
	}
}

#[no_mangle]
pub extern "C" fn i210_hw_set_rx_mode(driver: *mut I210Driver, mode: u32) {
	if let Some(drv) = driver_from_ptr(driver) {
		unsafe {
			drv.set_rx_mode(mode);
		}
	}
}
