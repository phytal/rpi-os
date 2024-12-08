// gic.c
#include "gic.h"

void gic_init() {
    // Disable all interrupts
    *((volatile uint32_t*)(IRQ_DISABLE_1)) = 0xFFFFFFFF;
    *((volatile uint32_t*)(IRQ_DISABLE_2)) = 0xFFFFFFFF;
    *((volatile uint32_t*)(IRQ_DISABLE_BASIC)) = 0xFFFFFFFF;

    // Clear all pending interrupts
    *((volatile uint32_t*)(IRQ_BASIC_PENDING)) = 0xFFFFFFFF;
    *((volatile uint32_t*)(IRQ_PENDING_1)) = 0xFFFFFFFF;
    *((volatile uint32_t*)(IRQ_PENDING_2)) = 0xFFFFFFFF;
}

void gic_enable_interrupt(uint32_t interrupt_id) {
    if (interrupt_id < 32) {
        *((volatile uint32_t*)(IRQ_ENABLE_1)) |= (1 << interrupt_id);
    } else if (interrupt_id < 64) {
        *((volatile uint32_t*)(IRQ_ENABLE_2)) |= (1 << (interrupt_id - 32));
    } else {
        *((volatile uint32_t*)(IRQ_ENABLE_BASIC)) |= (1 << (interrupt_id - 64));
    }
}

void gic_disable_interrupt(uint32_t interrupt_id) {
    if (interrupt_id < 32) {
        *((volatile uint32_t*)(IRQ_DISABLE_1)) = (1 << interrupt_id);
    } else if (interrupt_id < 64) {
        *((volatile uint32_t*)(IRQ_DISABLE_2)) = (1 << (interrupt_id - 32));
    } else {
        *((volatile uint32_t*)(IRQ_DISABLE_BASIC)) = (1 << (interrupt_id - 64));
    }
}