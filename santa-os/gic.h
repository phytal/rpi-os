// gic.h
#ifndef GIC_H
#define GIC_H

// BCM2837 legacy interrupt controller. The register addresses come from
// irq.h; this header used to redefine them as plain integers, which produced
// "redefined" warnings and made the meaning depend on include order.
#include "irq.h"

#define IRQ_FIQ_CONTROL     ((volatile uint32_t*)(IRQ_BASE+0x0C))
#define IRQ_DISABLE_BASIC   ((volatile uint32_t*)(IRQ_BASE+0x24))

void gic_init(void);
void gic_enable_interrupt(uint32_t interrupt_id);
void gic_disable_interrupt(uint32_t interrupt_id);

#endif
