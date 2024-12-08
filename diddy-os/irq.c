// irq.c
#include "irq.h"
#include "uart.h"
#include "timer.h"

extern void vectors_start(void);
static void (*irq_handlers[64])(void);

// irq.c 
void irq_handler(void) __attribute__((alias("handle_irq")));

void irq_init(void) {
    for (int i = 0; i < 64; i++) {
        irq_handlers[i] = 0;
    }
}

void irq_vector_init(void) {
    asm volatile("msr vbar_el1, %0" :: "r"(vectors_start));
}

void enable_irq(void) {
    asm volatile("msr daifclr, #2");
}

void disable_irq(void) {
    asm volatile("msr daifset, #2");
}

void register_irq_handler(int irq, void (*handler)(void)) {
    irq_handlers[irq] = handler;
}

void enable_interrupt(int irq) {
    volatile uint32_t *irq_enable = (uint32_t *)IRQ_ENABLE_1;
    irq_enable[irq / 32] = 1 << (irq % 32);
}

void disable_interrupt(int irq) {
    volatile uint32_t *irq_disable = (uint32_t *)IRQ_DISABLE_1;
    irq_disable[irq / 32] = 1 << (irq % 32);
}

void handle_irq(void) {
    printf("IRQ Handler Entry\n");
    
    uint32_t pending = *IRQ_PENDING_1;
    printf("Pending IRQs: 0x%x\n", pending);
    
    // Check timer interrupt specifically
    if (pending & (1 << 1)) {
        printf("Timer interrupt detected\n");
        if (irq_handlers[1]) {
            irq_handlers[1]();
        }
    }
    
    printf("IRQ Handler Exit\n");
}

void handle_exception(unsigned long esr, unsigned long elr, unsigned long spsr) {
    uart_puts("Exception occurred!\n");
    uart_puts("ESR: ");
    uart_hex(esr);
    uart_puts("\nELR: ");
    uart_hex(elr);
    uart_puts("\nSPSR: ");
    uart_hex(spsr);
    uart_puts("\n");
    
    while(1);
}