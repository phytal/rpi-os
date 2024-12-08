// #include "uart.h"
// #include "mbox.h"
// #include "sprintf.h"
// #include "heap.h"
// #include "utils.h"
// #include "sd.h"
// #include "delays.h"

// extern unsigned char _end;

// void kernel_main()
// {
// 	// set up serial console
//     uart_init();

//     int el = get_el();
//     printf("what is my exception level??? %d\n", el);
    
    
//     int HEAP_START = 1 * 1024 * 1024; // 1 MB
//     int HEAP_SIZE = 5 * 1024 * 1024;
//     heapInit((void*)HEAP_START,HEAP_SIZE);

//     uint32_t* ptr = malloc(17);
//     printf("ptr: %x\n", ptr);
//     *ptr = 4;
//     printf("ptr value: %d\n", *ptr);
//     free(ptr);

//     // if(sd_init()==SD_OK) {
//     //     // read the master boot record after our bss segment
//     //     if(sd_readblock(0,&_end,1)) {
//     //         // dump it to serial console
//     //         uart_puts(&_end);
//     //     }
//     // }

//     //heap_init();
    
//     // // get the board's unique serial number with a mailbox call
//     // mbox[0] = 8*4;                  // length of the message
//     // mbox[1] = MBOX_REQUEST;         // this is a request message
    
//     // mbox[2] = MBOX_TAG_GETSERIAL;   // get serial number command
//     // mbox[3] = 8;                    // buffer size
//     // mbox[4] = 8;
//     // mbox[5] = 0;                    // clear output buffer
//     // mbox[6] = 0;

//     // mbox[7] = MBOX_TAG_LAST;

//     // send the message to the GPU and receive answer
//     // if (mbox_call(MBOX_CH_PROP)) {
//     //     uart_puts("My serial number is: ");
//     //     uart_hex(mbox[6]);
//     //     uart_hex(mbox[5]);
//     //     uart_puts("\n");
//     // } else {
//     //     uart_puts("Unable to query serial!\n");
//     // }
// 	printf("Hello world!\n");

//     // echo everything back
//     while(1) {
//         uart_send(uart_getc());
//     }
// }

#include "uart.h"
#include "mbox.h"
#include "sprintf.h"
#include "gic.h"
#include "uart.h"
#include "timer.h"
#include "irq.h"

// UART registers
#define UART0_BASE      (MMIO_BASE + 0x201000)
#define UART0_DR        ((volatile uint32_t*)(UART0_BASE + 0x00))
#define UART0_FR        ((volatile uint32_t*)(UART0_BASE + 0x18))
#define UART0_ICR       ((volatile uint32_t*)(UART0_BASE + 0x44))
#define UART0_IMSC      ((volatile uint32_t*)(UART0_BASE + 0x38))

#define MMIO_BASE       0x3F000000

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

void kernel_main() {
    // Verify we're in EL1
    uint64_t current_el;
    asm volatile("mrs %0, CurrentEL" : "=r"(current_el));
    current_el = current_el >> 2;
    printf("Running at EL%d\n", current_el);

    // Only continue if we're in EL1
    if (current_el != 2) {
        printf("Error: Must be in EL2!\n");
        return;
    }

    // Set up vector table with debug
    uint64_t vbar;
    asm volatile("msr vbar_el2, %0" :: "r"(vectors_start));
    asm volatile("mrs %0, vbar_el2" : "=r"(vbar));
    printf("Vector table at: 0x%x\n", vbar);
    
    uart_init();

    printf("Initializing interrupts...\n");
    gic_init();
    irq_init();
    irq_vector_init();

    printf("Registering handlers...\n");
    register_irq_handler(IRQ_TIMER1, timer_irq_handler);
    register_irq_handler(IRQ_UART, uart_irq_handler);

    printf("Initializing timer...\n");
    timer_init();  // Move timer init before enabling interrupts

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
}