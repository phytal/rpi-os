// timer.c
#include "irq.h"
#include "timer.h"
#include "uart.h"

static volatile uint32_t system_timer_ticks = 0;

void timer_init(void) {
    printf("Timer init: Clearing interrupts...\n");
    *TIMER_CS = 0xF;  // Clear all timer interrupt flags
    
    printf("Timer init: Getting current time...\n");
    uint32_t current = *TIMER_CLO;
    printf("Current time: %u\n", current);
    
    // Set next interrupt in 1 second (1MHz clock = 1,000,000 ticks)
    uint32_t next = current + 1000000;
    printf("Setting next interrupt at: %u\n", next);
    *TIMER_C1 = next;
    
    // Enable timer match detection in control register
    *TIMER_CS |= (1 << 1);  // Enable match detection for C1
    
    printf("Timer control status: 0x%x\n", *TIMER_CS);
}

void timer_set(uint32_t value) {
    *TIMER_C1 = value;
}

uint32_t timer_get(void) {
    return *TIMER_CLO;
}

void timer_enable_interrupt(void) {
    printf("Enabling timer interrupt\n");
    *IRQ_ENABLE_1 |= (1 << 1);
}

void timer_disable_interrupt(void) {
    *IRQ_DISABLE_1 = (1 << 1); // Disable timer interrupt
}

void timer_irq_handler() {
    printf("Timer IRQ Handler Entry\n");
    
    // Clear the interrupt
    *TIMER_CS = (1 << 1);
    
    system_timer_ticks++;
    printf("System ticks: %u\n", system_timer_ticks);
    
    // Set next interrupt
    uint32_t current = *TIMER_CLO;
    uint32_t next = current + 1000000;
    *TIMER_C1 = next;
    printf("Next interrupt at: %u\n", next);
}