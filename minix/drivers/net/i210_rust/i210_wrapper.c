/* I210/I211 Ethernet Controller Driver Wrapper for MINIX */

#include <minix/drivers.h>
#include <minix/netdriver.h>
#include <machine/pci.h>
#include <sys/mman.h>
#include <errno.h>
#include <string.h>
#include "i210_wrapper.h"

#define I210_VENDOR_ID	0x8086
#define I210_DEVICE_I210	0x1539
#define I210_DEVICE_I211	0x153a

#define I210_RXDESC_NR	256
#define I210_TXDESC_NR	256
#define I210_IOBUF_SIZE	2048
#define I210_DESC_SIZE	16

typedef int irq_hook_t;

struct i210_device {
	unsigned int instance;
	int devind;
	int irq;
	int irq_hook;
	int irq_inuse;
	void *regs;
	size_t regs_size;
	struct i210_dma_block rx_desc;
	struct i210_dma_block rx_buf;
	struct i210_dma_block tx_desc;
	struct i210_dma_block tx_buf;
	size_t rx_count;
	size_t tx_count;
};

static struct i210_device i210_dev;
static void *i210_driver;

static int i210_netdriver_init(unsigned int instance, netdriver_addr_t *addr,
	uint32_t *caps, unsigned int *ticks);
static void i210_stop(void);
static void i210_set_mode(unsigned int mode,
	const netdriver_addr_t *mcast, unsigned int mcast_count);
static void i210_set_hwaddr(const netdriver_addr_t *addr);
static int i210_send(struct netdriver_data *data, size_t size);
static ssize_t i210_recv(struct netdriver_data *data, size_t max);
static unsigned int i210_get_link(uint32_t *media);
static void i210_intr(unsigned int mask);
static void i210_tick(void);

static int i210_probe(unsigned int instance);
static int i210_setup_dma(void);
static void i210_free_dma(void);
static int i210_request_irqs(void);
static void i210_release_irqs(void);
static void i210_cleanup(void);

static const struct netdriver i210_table = {
	.ndr_name	= "i210",
	.ndr_init	= i210_netdriver_init,
	.ndr_stop	= i210_stop,
	.ndr_set_mode	= i210_set_mode,
	.ndr_set_hwaddr	= i210_set_hwaddr,
	.ndr_recv	= i210_recv,
	.ndr_send	= i210_send,
	.ndr_get_link	= i210_get_link,
	.ndr_intr	= i210_intr,
	.ndr_tick	= i210_tick
};

static int
i210_match_device(u16_t vid, u16_t did)
{
	if (vid != I210_VENDOR_ID)
		return FALSE;
	return (did == I210_DEVICE_I210 || did == I210_DEVICE_I211);
}

static int
i210_probe(unsigned int instance)
{
	u16_t vid, did;
	int r, devind;
	u32_t size;
	int ioflag;
	u32_t base;
	u16_t cr;

	memset(&i210_dev, 0, sizeof(i210_dev));
	i210_dev.instance = instance;
	i210_dev.irq_hook = -1;

	pci_init();

	r = pci_first_dev(&devind, &vid, &did);
	if (!r)
		return ENXIO;

	while (TRUE) {
		if (i210_match_device(vid, did)) {
			if (instance == 0)
				break;
			instance--;
		}

		if (!(r = pci_next_dev(&devind, &vid, &did)))
			return ENXIO;
	}

	i210_dev.devind = devind;
	i210_dev.irq = pci_attr_r8(devind, PCI_ILR);

	pci_reserve(devind);

	if ((r = pci_get_bar(devind, PCI_BAR, &base, &size, &ioflag)) != OK)
		return r;
	if (ioflag)
		return EINVAL;

	i210_dev.regs = vm_map_phys(SELF, (void *)base, size);
	if (i210_dev.regs == MAP_FAILED)
		return ENOMEM;
	i210_dev.regs_size = size;

	cr = pci_attr_r16(devind, PCI_CR);
	if (!(cr & PCI_CR_MAST_EN))
		pci_attr_w16(devind, PCI_CR, cr | PCI_CR_MAST_EN);

	return OK;
}

static int
i210_setup_dma(void)
{
	phys_bytes phys;
	size_t size;

	i210_dev.rx_count = I210_RXDESC_NR;
	i210_dev.tx_count = I210_TXDESC_NR;

	size = I210_RXDESC_NR * I210_DESC_SIZE;
	i210_dev.rx_desc.size = size;
	i210_dev.rx_desc.virt = alloc_contig(size, AC_ALIGN4K, &phys);
	if (i210_dev.rx_desc.virt == NULL)
		goto fail;
	i210_dev.rx_desc.phys = phys;

	size = I210_RXDESC_NR * I210_IOBUF_SIZE;
	i210_dev.rx_buf.size = size;
	i210_dev.rx_buf.virt = alloc_contig(size, AC_ALIGN4K, &phys);
	if (i210_dev.rx_buf.virt == NULL)
		goto fail;
	i210_dev.rx_buf.phys = phys;

	size = I210_TXDESC_NR * I210_DESC_SIZE;
	i210_dev.tx_desc.size = size;
	i210_dev.tx_desc.virt = alloc_contig(size, AC_ALIGN4K, &phys);
	if (i210_dev.tx_desc.virt == NULL)
		goto fail;
	i210_dev.tx_desc.phys = phys;

	size = I210_TXDESC_NR * I210_IOBUF_SIZE;
	i210_dev.tx_buf.size = size;
	i210_dev.tx_buf.virt = alloc_contig(size, AC_ALIGN4K, &phys);
	if (i210_dev.tx_buf.virt == NULL)
		goto fail;
	i210_dev.tx_buf.phys = phys;

	return OK;

fail:
	i210_free_dma();
	return ENOMEM;
}

static void
i210_free_dma(void)
{
	if (i210_dev.rx_desc.virt)
		free_contig(i210_dev.rx_desc.virt, i210_dev.rx_desc.size);
	if (i210_dev.rx_buf.virt)
		free_contig(i210_dev.rx_buf.virt, i210_dev.rx_buf.size);
	if (i210_dev.tx_desc.virt)
		free_contig(i210_dev.tx_desc.virt, i210_dev.tx_desc.size);
	if (i210_dev.tx_buf.virt)
		free_contig(i210_dev.tx_buf.virt, i210_dev.tx_buf.size);

	memset(&i210_dev.rx_desc, 0, sizeof(i210_dev.rx_desc));
	memset(&i210_dev.rx_buf, 0, sizeof(i210_dev.rx_buf));
	memset(&i210_dev.tx_desc, 0, sizeof(i210_dev.tx_desc));
	memset(&i210_dev.tx_buf, 0, sizeof(i210_dev.tx_buf));
}

static int
i210_request_irqs(void)
{
	int r;

	i210_dev.irq_hook = i210_dev.irq;

	r = sys_irqsetpolicy(i210_dev.irq, 0, &i210_dev.irq_hook);
	if (r != OK)
		return r;

	r = sys_irqenable(&i210_dev.irq_hook);
	if (r != OK)
		return r;

	i210_dev.irq_inuse = TRUE;

	return OK;
}

static void
i210_release_irqs(void)
{
	if (!i210_dev.irq_inuse)
		return;

	sys_irqdisable(&i210_dev.irq_hook);
	sys_irqrmpolicy(&i210_dev.irq_hook);
	i210_dev.irq_inuse = FALSE;
}

static void
i210_cleanup(void)
{
	if (i210_driver)
		i210_hw_disable_interrupts(i210_driver);

	i210_release_irqs();

	if (i210_driver) {
		i210_hw_stop(i210_driver);
		i210_hw_destroy(i210_driver);
		i210_driver = NULL;
	}

	i210_free_dma();

	if (i210_dev.regs && i210_dev.regs_size)
		vm_unmap_phys(SELF, i210_dev.regs, i210_dev.regs_size);
	memset(&i210_dev, 0, sizeof(i210_dev));
}

static int
i210_netdriver_init(unsigned int instance, netdriver_addr_t *addr,
	uint32_t *caps, unsigned int *ticks)
{
	int r;

	if ((r = i210_probe(instance)) != OK)
		return r;

	i210_driver = i210_hw_create(i210_dev.regs);
	if (i210_driver == NULL) {
		i210_cleanup();
		return ENOMEM;
	}

	if ((r = i210_setup_dma()) != OK) {
		i210_cleanup();
		return r;
	}

	r = i210_hw_setup_dma(i210_driver,
	    &i210_dev.rx_desc, &i210_dev.rx_buf,
	    &i210_dev.tx_desc, &i210_dev.tx_buf,
	    i210_dev.rx_count, i210_dev.tx_count);
	if (r != OK) {
		i210_cleanup();
		return r;
	}

	i210_hw_reset(i210_driver);
	i210_hw_start(i210_driver);

	if ((r = i210_request_irqs()) != OK) {
		i210_cleanup();
		return r;
	}

	i210_hw_enable_interrupts(i210_driver);

	i210_hw_read_mac(i210_driver, addr->na_addr);

	*caps = 0;
	*ticks = sys_hz();

	return OK;
}

static void
i210_stop(void)
{
	i210_cleanup();
}

static void
i210_set_mode(unsigned int mode, const netdriver_addr_t *mcast,
	unsigned int mcast_count)
{
	(void)mcast;
	(void)mcast_count;
	if (!i210_driver)
		return;

	i210_hw_set_rx_mode(i210_driver, mode);
}

static void
i210_set_hwaddr(const netdriver_addr_t *addr)
{
	if (!i210_driver)
		return;

	i210_hw_set_mac(i210_driver, addr->na_addr);
}

static int
i210_send(struct netdriver_data *data, size_t size)
{
	struct i210_buffer_view view;
	int r;

	if (!i210_driver)
		return EIO;

	r = i210_hw_get_tx_buffer(i210_driver, &view);
	if (r == EAGAIN || r == EBUSY)
		return SUSPEND;
	if (r != OK)
		return r;

	if (size > view.len)
		return EINVAL;

	netdriver_copyin(data, 0, view.ptr, size);

	r = i210_hw_commit_tx(i210_driver, size);
	if (r == EAGAIN || r == EBUSY)
		return SUSPEND;
	return r;
}

static ssize_t
i210_recv(struct netdriver_data *data, size_t max)
{
	struct i210_buffer_view view;
	ssize_t len;
	int r;

	if (!i210_driver)
		return EIO;

	r = i210_hw_peek_rx(i210_driver, &view);
	if (r == EAGAIN || r == EBUSY)
		return SUSPEND;
	if (r != OK)
		return r;

	len = (ssize_t)view.len;
	if (len < NDEV_ETH_PACKET_MIN)
		len = NDEV_ETH_PACKET_MIN;
	if ((size_t)len > max)
		len = (ssize_t)max;

	netdriver_copyout(data, 0, view.ptr, len);
	i210_hw_release_rx(i210_driver);

	return len;
}

static unsigned int
i210_get_link(uint32_t *media)
{
	if (!i210_driver)
		return NDEV_LINK_DOWN;

	if (i210_hw_link_status(i210_driver)) {
		*media = IFM_ETHER | IFM_1000_T | IFM_FDX;
		return NDEV_LINK_UP;
	}

	*media = IFM_ETHER;
	return NDEV_LINK_DOWN;
}

static void
i210_intr(unsigned int mask)
{
	(void)mask;
	uint32_t events;

	if (!i210_driver)
		return;

	events = i210_hw_handle_interrupts(i210_driver);

	if (events & I210_EVENT_LINK)
		netdriver_link();
	if (events & I210_EVENT_RX)
		netdriver_recv();
	if (events & I210_EVENT_TX)
		netdriver_send();

	sys_irqenable(&i210_dev.irq_hook);
}

static void
i210_tick(void)
{
	if (i210_driver)
		netdriver_link();
}

int
main(int argc, char *argv[])
{
	env_setargs(argc, argv);
	netdriver_task(&i210_table);
	return 0;
}