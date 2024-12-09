// gic.h
#ifndef GIC_H
#define GIC_H

#include "stdint.h"

// BCM2837 Interrupt Controller
#define MMIO_BASE       0x3F000000
#define IRQ_BASE        (MMIO_BASE + 0x0000B200)

// IRQ Controller registers
#define IRQ_BASIC_PENDING   (IRQ_BASE + 0x00)
#define IRQ_PENDING_1       (IRQ_BASE + 0x04)
#define IRQ_PENDING_2       (IRQ_BASE + 0x08)
#define IRQ_FIQ_CONTROL    (IRQ_BASE + 0x0C)
#define IRQ_ENABLE_1       (IRQ_BASE + 0x10)
#define IRQ_ENABLE_2       (IRQ_BASE + 0x14)
#define IRQ_ENABLE_BASIC   (IRQ_BASE + 0x18)
#define IRQ_DISABLE_1      (IRQ_BASE + 0x1C)
#define IRQ_DISABLE_2      (IRQ_BASE + 0x20)
#define IRQ_DISABLE_BASIC  (IRQ_BASE + 0x24)

void gic_init(void);
void gic_enable_interrupt(uint32_t interrupt_id);
void gic_disable_interrupt(uint32_t interrupt_id);

#endif