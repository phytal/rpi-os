#ifndef TIMER_H
#define TIMER_H

#include "stdint.h"

#define MMIO_BASE       0x3F000000
#define TIMER_BASE      (MMIO_BASE + 0x003000)
#define TIMER_CS        ((volatile uint32_t*)(TIMER_BASE + 0x00))
#define TIMER_CLO       ((volatile uint32_t*)(TIMER_BASE + 0x04))
#define TIMER_CHI       ((volatile uint32_t*)(TIMER_BASE + 0x08))
#define TIMER_C0        ((volatile uint32_t*)(TIMER_BASE + 0x0C))
#define TIMER_C1        ((volatile uint32_t*)(TIMER_BASE + 0x10))
#define TIMER_C2        ((volatile uint32_t*)(TIMER_BASE + 0x14))
#define TIMER_C3        ((volatile uint32_t*)(TIMER_BASE + 0x18))

void timer_init(void);
void timer_set(uint32_t value);
uint32_t timer_get(void);
void timer_enable_interrupt(void);
void timer_disable_interrupt(void);
void timer_irq_handler(void);

#endif