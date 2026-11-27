#include "stdint.h"
#include "uart.h"
#include "irq.h"
#include "timer.h"
#include "gic.h"
#include "heap.h"
#include "ext2.h"
#include "utils.h"
#include "constants.h"

// Barriers and events used for core synchronization
static inline void dmb(void) {
    asm volatile("dmb sy");
}

static inline void dsb(void) {
    asm volatile("dsb sy");
}

static inline void sev(void) {
    asm volatile("sev");
}

static inline void wfe(void) {
    asm volatile("wfe");
}

void kmain(void) __attribute__((noreturn));
void kmain2(void);

// ARM system register access
#define READ_SYSTEM_REG(reg) ({ \
    uint64_t _val; \
    __asm__ volatile("mrs %0, " #reg : "=r"(_val)); \
    _val; \
})

static inline uint64_t current_el(void) {
    return (READ_SYSTEM_REG(CurrentEL) >> 2) & 0x3;
}

// Called from boot.S on core 0, before kinit().
void zeros_bss(void) {
    extern uint64_t __bss_beg, __bss_end;
    uint64_t *bss = &__bss_beg;
    while (bss < &__bss_end) {
        *bss++ = 0;
    }
}

// Bit n is set once core n has started.
volatile uint32_t cores_ready = 0;

// Cores 1-3 land here from boot.S:secondary_start, which already gave them
// a stack, EL1 and VBAR_EL1.
void __attribute__((noreturn)) start2(void) {
    kmain2();
    while(1) {
        wfe();
    }
}

// The firmware (and QEMU) park cores 1-3 on this spin table: writing an
// address to entry n releases core n to that address.
#define SPIN_TABLE ((volatile uint64_t*)0xd8)
extern void secondary_start(void);

void initialize_app_cores(void) {
    printf("Initializing secondary cores...\n");

    // One core at a time: printf() shares a single buffer at _end, and
    // without the MMU/caches on we can't rely on exclusive-access spinlocks.
    for (int core = 1; core < NCORES; core++) {
        printf("Starting core %d\n", core);
        SPIN_TABLE[core] = (uint64_t)secondary_start;
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
    dmb();
    cores_ready |= (1 << core_num);
    dsb();
    sev();
    while(1) {
        wfe();
    }
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

// Core 0, called from boot.S after core_setup (stack, EL1, vector table),
// zeros_bss and uart_init.
void __attribute__((noreturn)) kinit(void) {
    uart_puts("Starting initialization...\n");
    printf("Current EL: %d\n", (int)current_el());

    uint64_t vbar;
    asm volatile("mrs %0, vbar_el1" : "=r"(vbar));
    printf("Vector table at: 0x%lx\n", vbar);

    heapInit((void*)HEAP_START, HEAP_SIZE);
    heap_tests();

    printf("| Initializing file system... \n");
    initialize_FS();

    printf("Initializing interrupts...\n");
    gic_init();
    irq_init();

    printf("Registering handlers...\n");
    register_irq_handler(IRQ_TIMER1, timer_irq_handler);
    register_irq_handler(IRQ_UART, uart_irq_handler);

    printf("Initializing timer...\n");
    timer_init();

    printf("Enabling interrupts...\n");
    enable_interrupt(IRQ_TIMER1);
    enable_interrupt(IRQ_UART);
    enable_irq();               // CPU interrupts last

    uart_puts("Jumping to kmain\n");
    kmain();
}
