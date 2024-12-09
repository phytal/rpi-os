#include "uart.h"
#include "sprintf.h"
#include "gic.h"
#include "uart.h"
#include "timer.h"
#include "irq.h"
#include "sd.h"
#include "heap.h"
#include "gpio.h"
#include "ext2.h"
// #include "utils.h"

// UART registers
#define UART0_BASE      (MMIO_BASE + 0x201000)
#define UART0_DR        ((volatile uint32_t*)(UART0_BASE + 0x00))
#define UART0_FR        ((volatile uint32_t*)(UART0_BASE + 0x18))
#define UART0_ICR       ((volatile uint32_t*)(UART0_BASE + 0x44))
#define UART0_IMSC      ((volatile uint32_t*)(UART0_BASE + 0x38))

#define MMIO_BASE       0x3F000000

int HEAP_START = 1 * 1024 * 1024; // 1 MB
int HEAP_SIZE = 5 * 1024 * 1024;

// Vector table setup in assembly
extern void vectors_start(void);

void uart_irq_handler() {
    // Read UART flags register
    uint32_t status = *UART0_FR;
    
    // While receive FIFO is not empty
    while (!(status & (1 << 4))) {
        // Read and echo character
        char c = (char)(*UART0_DR);
        uart_send(c);
        
        // Check if more data available
        status = *UART0_FR;
    }
    
    // Clear all UART interrupts
    *UART0_ICR = 0x7FF;
}

void *dumcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char*) dest;
    const unsigned char *s = (const unsigned char*) src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}

void kernel_main() {
    // Verify we're in EL2
    uint64_t current_el;
    asm volatile("mrs %0, CurrentEL" : "=r"(current_el));
    current_el = current_el >> 2;
    printf("Running at EL%d\n", current_el);

    // Only continue if we're in EL2
    if (current_el != 2) {
        printf("Error: Must be in EL2!\n");
        return;
    }

    // Set up vector table with debug
    uint64_t vbar;
    asm volatile("msr vbar_el2, %0" :: "r"(vectors_start));
    asm volatile("mrs %0, vbar_el2" : "=r"(vbar));
    printf("Vector table at: 0x%x\n", vbar);
    
    // initialize uart ports
    uart_init();

    // initialize heap
    heapInit((void*)HEAP_START,HEAP_SIZE);

    printf("<------------ HEAP TESTS ------------>\n");
    int* num1 = malloc(sizeof(int));
    int* num2 = malloc(sizeof(int));

    *num1 = 15;
    *num2 = 18;
    printf("before bum1 and bum2: %d, %d\n", *num1, *num2);
    dumcpy(num2, num1, 4);
    printf("after bum1 and bum2: %d, %d\n", *num1, *num2);
    printf("<------------ END OF HEAP TESTS ------------>\n\n");

    // initialize ext2
    printf("| Initializing file system... \n");
    initialize_FS();

    printf("Initializing interrupts...\n");
    gic_init();
    irq_init();
    irq_vector_init();

    printf("Registering handlers...\n");
    register_irq_handler(IRQ_TIMER1, timer_irq_handler);
    register_irq_handler(IRQ_UART, uart_irq_handler);

    printf("Initializing timer...\n");
    timer_init();  // Move timer init before enabling interrupts

    // everything below is interrupt stuff
    printf("Enabling interrupts...\n");
    enable_interrupt(IRQ_TIMER1);
    enable_interrupt(IRQ_UART);
    
    // Enable CPU interrupts last
    enable_irq();
    asm volatile("msr daifclr, #2"); 

    printf("System state verification:\n");
    printf("TIMER_CS: 0x%x\n", *TIMER_CS);
    printf("IRQ_ENABLE_1: 0x%x\n", *IRQ_ENABLE_1);
    printf("IRQ_PENDING_1: 0x%x\n", *IRQ_PENDING_1);
    
    printf("System running!\n");
    
    while(1) {
        uint32_t cs = *TIMER_CS;
        uint32_t pending = *IRQ_PENDING_1;
        if (cs || pending) {
            // printf("CS: 0x%x, PENDING: 0x%x\n", cs, pending);
        }
        asm volatile("wfe");
    }

    // while(1) {
    // }
}