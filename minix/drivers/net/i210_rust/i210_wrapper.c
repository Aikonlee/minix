/* I210/I211 Ethernet Controller Driver Wrapper for MINIX */

#include <minix/drivers.h>
#include <minix/netdriver.h>
#include <machine/pci.h>
#include <sys/mman.h>
#include <assert.h>
#include <string.h>
#include "i210_wrapper.h"

// FFI declarations for Rust functions
extern void *i210_init(void *base_addr);
extern void i210_reset(void *driver);
extern void i210_read_mac(void *driver, unsigned char *mac);
extern int i210_check_link(void *driver);
extern void i210_enable_rx(void *driver);
extern void i210_enable_tx(void *driver);

static int i210_instance;
static void *i210_driver;

static int i210_init(unsigned int instance, netdriver_addr_t *addr,
	uint32_t *caps, unsigned int *ticks);
static void i210_stop(void);
static void i210_set_mode(unsigned int mode, const netdriver_addr_t *mcast,
	unsigned int mcast_count);
static void i210_set_hwaddr(const netdriver_addr_t *addr);
static int i210_send(struct netdriver_data *data, size_t size);
static ssize_t i210_recv(struct netdriver_data *data, size_t max);
static unsigned int i210_get_link(uint32_t *media);
static void i210_intr(unsigned int mask);
static void i210_tick(void);
static int i210_probe(int skip);

static const struct netdriver i210_table = {
	.ndr_name	= "i210",
	.ndr_init	= i210_init,
	.ndr_stop	= i210_stop,
	.ndr_set_mode	= i210_set_mode,
	.ndr_set_hwaddr	= i210_set_hwaddr,
	.ndr_recv	= i210_recv,
	.ndr_send	= i210_send,
	.ndr_get_link	= i210_get_link,
	.ndr_intr	= i210_intr,
	.ndr_tick	= i210_tick
};

static int i210_init(unsigned int instance, netdriver_addr_t *addr,
	uint32_t *caps, unsigned int *ticks)
{
	int r;
	u16_t vid, did;
	u32_t bar;
	size_t size;
	void *base_addr;

	i210_instance = instance;

	/* Try to detect the device. */
	if ((r = i210_probe(0)) != OK)
		return r;

	/* Get the PCI BAR. */
	if ((r = pci_get_bar(PCI_BAR, &bar, &size)) != OK)
		return r;

	/* Map the memory. */
	if ((base_addr = vm_map_phys(SELF, (void *) bar, size)) == MAP_FAILED)
		return ENOMEM;

	/* Initialize the Rust driver. */
	i210_driver = i210_init(base_addr);

	/* Read the MAC address. */
	i210_read_mac(i210_driver, addr->na_addr);

	/* Set capabilities. */
	*caps = NETIF_CAP_BROADCAST | NETIF_CAP_ARP | NETIF_CAP_IPV4 | NETIF_CAP_IPV6;

	/* Set tick interval. */
	*ticks = 1; /* 1 tick per second */

	return OK;
}

static void i210_stop(void)
{
	/* Reset the device. */
	if (i210_driver)
		i210_reset(i210_driver);
}

static void i210_set_mode(unsigned int mode, const netdriver_addr_t *mcast,
	unsigned int mcast_count)
{
	/* Mode setting not implemented yet. */
}

static void i210_set_hwaddr(const netdriver_addr_t *addr)
{
	/* Hardware address setting not implemented yet. */
}

static int i210_send(struct netdriver_data *data, size_t size)
{
	/* Send implementation not complete yet. */
	return ENOTSUP;
}

static ssize_t i210_recv(struct netdriver_data *data, size_t max)
{
	/* Receive implementation not complete yet. */
	return 0;
}

static unsigned int i210_get_link(uint32_t *media)
{
	if (i210_driver && i210_check_link(i210_driver)) {
		*media = IFM_ETHER | IFM_100_TX | IFM_FDX;
		return NETIF_STATUS_UP;
	} else {
		*media = IFM_ETHER;
		return NETIF_STATUS_DOWN;
	}
}

static void i210_intr(unsigned int mask)
{
	/* Interrupt handling not implemented yet. */
}

static void i210_tick(void)
{
	/* Periodic tasks not implemented yet. */
}

static int i210_probe(int skip)
{
	u16_t vid, did;
	int r;

	if ((r = pci_find_dev(NULL, skip, &vid, &did, NULL)) != OK)
		return r;

	if (vid != 0x8086 || (did != 0x1539 && did != 0x153A)) /* I210/I211 */
		return ENXIO;

	return OK;
}

int main(int argc, char *argv[])
{
	env_setargs(argc, argv);
	netdriver_task(&i210_table);
	return 0;
}