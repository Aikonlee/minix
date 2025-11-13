/* Intel I210/I211 Network Driver Wrapper for MINIX3 */

#include <minix/drivers.h>
#include <minix/netdriver.h>
#include <machine/pci.h>
#include <sys/mman.h>
#include <assert.h>

#include "i210_rust.h"

/* PCI device IDs */
#define I210_VENDOR_ID      0x8086
#define I210_DEVICE_ID      0x1539  /* I210/I211 */

/* Driver constants */
#define I210_RXDESC_NR      256
#define I210_TXDESC_NR      256
#define I210_IOBUF_SIZE     2048

/* Driver state */
typedef struct i210_state {
    int irq;
    int irq_hook;
    u8_t *regs;
    u32_t regs_size;
    u8_t mac_addr[6];
} i210_state_t;

static int i210_instance;
static i210_state_t i210_state;

/* Function declarations for Rust FFI */
extern int i210_rust_init(u8_t *regs, int irq);
extern int i210_rust_get_mac_addr(u8_t *mac_addr);
extern int i210_rust_enable_rx(void);
extern int i210_rust_enable_tx(void);
extern int i210_rust_get_link_status(void);
extern int i210_rust_reset(void);

/* Forward declarations */
static int i210_init(unsigned int instance, netdriver_addr_t *addr,
    uint32_t *caps, unsigned int *ticks);
static void i210_stop(void);
static void i210_set_mode(unsigned int mode, const netdriver_addr_t *mcast_list,
    unsigned int mcast_count);
static void i210_set_hwaddr(const netdriver_addr_t *addr);
static int i210_send(struct netdriver_data *data, size_t size);
static ssize_t i210_recv(struct netdriver_data *data, size_t max);
static unsigned int i210_get_link(uint32_t *media);
static void i210_intr(unsigned int mask);
static void i210_tick(void);
static int i210_probe(i210_state_t *state, int skip);

/* NetDriver interface table */
static const struct netdriver i210_table = {
    .ndr_name       = "i210",
    .ndr_init       = i210_init,
    .ndr_stop       = i210_stop,
    .ndr_set_mode   = i210_set_mode,
    .ndr_set_hwaddr = i210_set_hwaddr,
    .ndr_recv       = i210_recv,
    .ndr_send       = i210_send,
    .ndr_get_link   = i210_get_link,
    .ndr_intr       = i210_intr,
    .ndr_tick       = i210_tick
};

/*
 * Main entry point
 */
int
main(int argc, char *argv[])
{
    env_setargs(argc, argv);
    netdriver_task(&i210_table);
    return 0;
}

/*
 * Probe for Intel I210/I211 device
 */
static int
i210_probe(i210_state_t *state, int skip)
{
    int r, devind;
    u16_t vid, did;
    u32_t bar;
    u8_t irq;

    pci_init();

    r = pci_first_dev(&devind, &vid, &did);
    while (r > 0) {
        if (vid == I210_VENDOR_ID && did == I210_DEVICE_ID) {
            if (skip == 0)
                break;
            skip--;
        }
        r = pci_next_dev(&devind, &vid, &did);
    }

    if (r <= 0)
        return FALSE;

    pci_reserve(devind);

    /* Get BAR0 (memory-mapped registers) */
    bar = pci_attr_r32(devind, PCI_BAR);
    state->regs_size = 0x20000; /* 128KB typical register space */
    
    /* Map registers */
    state->regs = (u8_t *)vm_map_phys(SELF, (void *)bar, state->regs_size);
    if (state->regs == MAP_FAILED) {
        panic("i210: failed to map registers");
    }

    /* Get IRQ */
    irq = pci_attr_r8(devind, PCI_ILR);
    state->irq = irq;
    state->irq_hook = state->irq;

    /* Enable PCI bus mastering */
    pci_set_acl(devind);

    return TRUE;
}

/*
 * Initialize the driver
 */
static int
i210_init(unsigned int instance, netdriver_addr_t *addr, uint32_t *caps,
    unsigned int *ticks)
{
    i210_state_t *state;
    int r;

    i210_instance = instance;

    /* Clear state */
    memset(&i210_state, 0, sizeof(i210_state));
    state = &i210_state;

    /* Calibrate TSC */
    if ((r = tsc_calibrate()) != OK)
        panic("tsc_calibrate failed: %d", r);

    /* Probe for device */
    if (!i210_probe(state, instance))
        return ENXIO;

    /* Initialize Rust driver */
    if (i210_rust_init(state->regs, state->irq) != 0) {
        return EIO;
    }

    /* Get MAC address */
    if (i210_rust_get_mac_addr(state->mac_addr) != 0) {
        return EIO;
    }

    /* Copy MAC address to output */
    memcpy(addr->na_addr, state->mac_addr, sizeof(state->mac_addr));

    /* Enable interrupts */
    if ((r = sys_irqsetpolicy(state->irq, 0, &state->irq_hook)) != OK)
        panic("sys_irqsetpolicy failed: %d", r);
    
    if ((r = sys_irqenable(&state->irq_hook)) != OK)
        panic("sys_irqenable failed: %d", r);

    /* Enable receiver and transmitter */
    i210_rust_enable_rx();
    i210_rust_enable_tx();

    *caps = NDEV_CAP_MCAST | NDEV_CAP_BCAST | NDEV_CAP_HWADDR;
    *ticks = sys_hz() / 10; /* 10 Hz */

    return OK;
}

/*
 * Stop the driver
 */
static void
i210_stop(void)
{
    i210_state_t *state = &i210_state;

    /* Disable interrupts */
    if (state->irq_hook != 0) {
        sys_irqdisable(&state->irq_hook);
        sys_irqrmpolicy(&state->irq_hook);
        state->irq_hook = 0;
    }

    /* Reset device */
    i210_rust_reset();
}

/*
 * Set receive mode
 */
static void
i210_set_mode(unsigned int mode, const netdriver_addr_t *mcast_list,
    unsigned int mcast_count)
{
    /* TODO: Implement multicast filtering */
}

/*
 * Set hardware address
 */
static void
i210_set_hwaddr(const netdriver_addr_t *addr)
{
    /* TODO: Implement MAC address override */
}

/*
 * Send a packet
 */
static int
i210_send(struct netdriver_data *data, size_t size)
{
    /* TODO: Implement packet transmission */
    return OK;
}

/*
 * Receive a packet
 */
static ssize_t
i210_recv(struct netdriver_data *data, size_t max)
{
    /* TODO: Implement packet reception */
    return SUSPEND;
}

/*
 * Get link status
 */
static unsigned int
i210_get_link(uint32_t *media)
{
    int link_status;

    link_status = i210_rust_get_link_status();

    if (link_status) {
        *media = IFM_ETHER | IFM_1000_T | IFM_FDX;
        return NDEV_LINK_UP;
    }

    *media = 0;
    return NDEV_LINK_DOWN;
}

/*
 * Handle interrupt
 */
static void
i210_intr(unsigned int mask)
{
    i210_state_t *state = &i210_state;

    /* Re-enable interrupt */
    sys_irqenable(&state->irq_hook);
}

/*
 * Periodic timer tick
 */
static void
i210_tick(void)
{
    /* Nothing to do for now */
}
