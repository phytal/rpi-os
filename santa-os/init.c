#include "stdint.h"
#include "sprintf.h"
#include "uart.h"
#include "irq.h"
#include "timer.h"
#include "gic.h"
#include "mmu.h"
#include "heap.h"
#include "ext2.h"
#include "utils.h"
#include "constants.h"

// Memory barrier for core synchronization  
static inline void dmb(void) {
    asm volatile("dmb sy");
}

// Instruction Synchronization Barrier
static inline void isb(void) {
    asm volatile("isb sy");
}

// Data Synchronization Barrier
static inline void dsb(void) {
    asm volatile("dsb sy");
}

static inline void wfe(void) {
    asm volatile("wfe");
}

// Forward declarations
void kinit(void);
void kinit2(void);
void kmain(void);
void kmain2(void);
void zeros_bss(void);
void initialize_app_cores(void);

// Memory-mapped base address for core synchronization
volatile uint64_t* const SPINNING_BASE = (volatile uint64_t*)0x50000;

// External symbols from linker script
extern uint64_t __bss_beg;
extern uint64_t __bss_end;
extern uint64_t vectors_start;

// ARM system register access macros
#define READ_SYSTEM_REG(reg) ({ \
    uint64_t _val; \
    __asm__ volatile("mrs %0, " #reg : "=r"(_val)); \
    _val; \
})

#define WRITE_SYSTEM_REG(reg, val) \
    __asm__ volatile("msr " #reg ", %0" :: "r"((uint64_t)(val)))

// ARM system instructions
static inline void sev(void) {
    __asm__ volatile("sev");
}

static inline uint64_t current_el(void) {
    return (READ_SYSTEM_REG(CurrentEL) >> 2) & 0x3;
}

// init.c
void zeros_bss(void) {
    extern uint64_t __bss_beg, __bss_end;
    uint64_t *bss = &__bss_beg;
    while (bss < &__bss_end) {
        *bss++ = 0;
    }
}

// Per-core initialization status
volatile uint32_t cores_ready = 0;

// init.h
void start2(void) __attribute__((noreturn));

void __attribute__((noreturn)) start2(void) {
    // boot.S:secondary_start already gave us a stack, EL1 and VBAR_EL1.
    kmain2();
    while(1) {
        wfe();
    }
}

#define SPINNING_BASE ((volatile uint64_t*)0x50000)

// Add to init.c
// #define QEMU_SPIN_TABLE 0xd8

// void initialize_app_cores(void) {
//     printf("Initializing secondary cores...\n");
    
//     // Map both spin tables
//     for (int core = 0; core < NCORES; core++) {
//         SPINNING_BASE[core] = 0;
//         ((volatile uint64_t*)QEMU_SPIN_TABLE)[core] = (uint64_t)start2;
//     }
//     dmb();
    
//     // Signal cores
//     for (int core = 1; core < NCORES; core++) {
//         printf("Signaling core %d\n", core);
//         dmb();
//         ((volatile uint64_t*)QEMU_SPIN_TABLE)[core] = (uint64_t)start2;
//         SPINNING_BASE[core] = (uint64_t)start2;  // Backup
//         dsb();
//         sev();
        
//         printf("Waiting for core %d\n", core);
//         while (!(cores_ready & (1 << core))) {
//             dmb();
//             asm volatile("wfe");
//         }
//     }
// }

#define QEMU_SPIN_TABLE 0xd8
extern void secondary_start(void);

void initialize_app_cores(void) {
    printf("Initializing secondary cores...\n");

    volatile uint64_t* spin_table = (volatile uint64_t*)QEMU_SPIN_TABLE;

    // One core at a time: printf() shares a single buffer at _end, and
    // without the MMU/caches on we can't rely on exclusive-access spinlocks.
    for (int core = 1; core < NCORES; core++) {
        printf("Starting core %d\n", core);
        spin_table[core] = (uint64_t)secondary_start;
        dsb();
        sev();
        while (!(cores_ready & (1 << core))) {
            dmb();
        }
    }
    printf("All %d cores running (ready mask 0x%x)\n", NCORES, cores_ready);
}

void kmain2(void) {
    uint64_t core_num = READ_SYSTEM_REG(MPIDR_EL1) & 0x3;
    printf("Hello from core %d! (EL%d)\n", (int)core_num, (int)current_el());
    SPINNING_BASE[core_num] = 0;
    dmb();
    cores_ready |= (1 << core_num);
    dsb();
    sev();
    while(1) {
        wfe();
    }
}

void set_vector_table(void) {
    extern uint64_t vectors_start;
    WRITE_SYSTEM_REG(VBAR_EL1, (uint64_t)&vectors_start);
    isb();  // Instruction synchronization barrier
}

// Heap smoke test carried over from banana-os/kernel.c.
static void heap_tests(void) {
    printf("<------------ HEAP TESTS ------------>\n");
    int* num1 = malloc(sizeof(int));
    int* num2 = malloc(sizeof(int));

    *num1 = 15;
    *num2 = 18;
    printf("before bum1 and bum2: %d, %d\n", *num1, *num2);
    memcpy(num2, num1, sizeof(int));
    printf("after bum1 and bum2: %d, %d\n", *num1, *num2);
    free(num1);
    free(num2);
    printf("<------------ END OF HEAP TESTS ------------>\n\n");
}

void __attribute__((noreturn)) kinit(void) {
    // Initialize UART first for debug output
    uart_init();
    uart_puts("Starting initialization...\n");
    
    // Get current exception level
    uint64_t el = current_el();
    printf("Current EL: %d\n", el);
    
    // Initialize core subsystems
    uart_puts("Initializing core subsystems\n");
    zeros_bss();

    // mmu_init();

    // Set up vector table with debug
    printf("Setting up vector table...\n");
    set_vector_table();
    uint64_t vbar;
    asm volatile("mrs %0, vbar_el1" : "=r"(vbar));
    printf("Vector table at: 0x%lx\n", vbar);
    
    uart_init();

    heapInit((void*)HEAP_START, HEAP_SIZE);
    heap_tests();

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

    printf("Enabling interrupts...\n");
    enable_interrupt(IRQ_TIMER1);
    enable_interrupt(IRQ_UART);
    
    // Enable CPU interrupts last
    enable_irq();
    
    // Continue to main kernel
    uart_puts("Jumping to kmain\n");
    kmain();
    
    while(1) { }
}