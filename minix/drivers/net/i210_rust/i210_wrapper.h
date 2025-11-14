/* I210/I211 Ethernet Controller Driver Wrapper Header */

#ifndef I210_WRAPPER_H
#define I210_WRAPPER_H

/* FFI declarations for Rust functions */
extern void *i210_init(void *base_addr);
extern void i210_reset(void *driver);
extern void i210_read_mac(void *driver, unsigned char *mac);
extern int i210_check_link(void *driver);
extern void i210_enable_rx(void *driver);
extern void i210_enable_tx(void *driver);

#endif /* I210_WRAPPER_H */