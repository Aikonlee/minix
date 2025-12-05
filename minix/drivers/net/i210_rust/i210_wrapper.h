/* I210/I211 Ethernet Controller Driver Wrapper Header */

#ifndef I210_WRAPPER_H
#define I210_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

struct i210_dma_block {
	void *virt;
	uint64_t phys;
	size_t size;
};

struct i210_buffer_view {
	void *ptr;
	size_t len;
};

enum {
	I210_EVENT_RX = 0x1,
	I210_EVENT_TX = 0x2,
	I210_EVENT_LINK = 0x4,
};

void *i210_hw_create(void *mmio_base);
void i210_hw_destroy(void *driver);
int i210_hw_setup_dma(void *driver,
	const struct i210_dma_block *rx_desc,
	const struct i210_dma_block *rx_buf,
	const struct i210_dma_block *tx_desc,
	const struct i210_dma_block *tx_buf,
	size_t rx_count,
	size_t tx_count);
void i210_hw_start(void *driver);
void i210_hw_stop(void *driver);
void i210_hw_reset(void *driver);
void i210_hw_read_mac(void *driver, unsigned char mac[6]);
int i210_hw_set_mac(void *driver, const unsigned char mac[6]);
int i210_hw_link_status(void *driver);
int i210_hw_get_tx_buffer(void *driver, struct i210_buffer_view *view);
int i210_hw_commit_tx(void *driver, size_t len);
int i210_hw_peek_rx(void *driver, struct i210_buffer_view *view);
void i210_hw_release_rx(void *driver);
uint32_t i210_hw_handle_interrupts(void *driver);
void i210_hw_enable_interrupts(void *driver);
void i210_hw_disable_interrupts(void *driver);
void i210_hw_set_rx_mode(void *driver, unsigned int mode);

#endif /* I210_WRAPPER_H */