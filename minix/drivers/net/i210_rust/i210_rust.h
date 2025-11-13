/**
 * @file i210_rust.h
 *
 * @brief Header file for Intel I210/I211 network driver
 *
 * This driver is implemented in Rust with a C wrapper for MINIX3 integration.
 * Supports Intel I210/I211 Gigabit Ethernet controllers (vendor:device = 8086:1539)
 */

#ifndef __I210_RUST_H
#define __I210_RUST_H

#include <minix/drivers.h>

/* Rust FFI function declarations */
extern int i210_rust_init(u8_t *regs, int irq);
extern int i210_rust_get_mac_addr(u8_t *mac_addr);
extern int i210_rust_enable_rx(void);
extern int i210_rust_enable_tx(void);
extern int i210_rust_get_link_status(void);
extern int i210_rust_reset(void);

#endif /* __I210_RUST_H */
