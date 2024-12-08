// irq.h
#ifndef IRQ_H
#define IRQ_H

#include "stdint.h"

// Exception levels
#define EL0 0
#define EL1 1
#define EL2 2
#define EL3 3

// Interrupt numbers
#define IRQ_TIMER1      1
#define IRQ_TIMER3      3
#define IRQ_USB         9
#define IRQ_GPIO0       49
#define IRQ_GPIO1       50
#define IRQ_GPIO2       51
#define IRQ_GPIO3       52
#define IRQ_UART        57

// Base addresses and registers
#define MMIO_BASE       0x3F000000
#define IRQ_BASE        (MMIO_BASE + 0x0000B200)
#define IRQ_BASIC_PENDING   ((volatile uint32_t*)(IRQ_BASE+0x00))
#define IRQ_PENDING_1       ((volatile uint32_t*)(IRQ_BASE+0x04))
#define IRQ_PENDING_2       ((volatile uint32_t*)(IRQ_BASE+0x08))
#define IRQ_ENABLE_1        ((volatile uint32_t*)(IRQ_BASE+0x10))
#define IRQ_ENABLE_2        ((volatile uint32_t*)(IRQ_BASE+0x14))
#define IRQ_ENABLE_BASIC    ((volatile uint32_t*)(IRQ_BASE+0x18))
#define IRQ_DISABLE_1       ((volatile uint32_t*)(IRQ_BASE+0x1C))
#define IRQ_DISABLE_2       ((volatile uint32_t*)(IRQ_BASE+0x20))

// Vector table offsets
#define SYNC_INVALID_EL1t   0x000
#define IRQ_INVALID_EL1t    0x080
#define FIQ_INVALID_EL1t    0x100
#define ERROR_INVALID_EL1t  0x180
#define SYNC_INVALID_EL1h   0x200
#define IRQ_INVALID_EL1h    0x280
#define FIQ_INVALID_EL1h    0x300
#define ERROR_INVALID_EL1h  0x380

// Function declarations
void irq_init(void);
void irq_vector_init(void);
void enable_irq(void);
void disable_irq(void);
void register_irq_handler(int irq, void (*handler)(void));
void enable_interrupt(int irq);
void disable_interrupt(int irq);
void handle_irq(void);

#endif