#include "stdint.h"
#include "sprintf.h"
#include "uart.h"
#include "irq.h"
#include "timer.h"
#include "gic.h"
#include "mmu.h"

#define NCORES 4
#define KERN_STACK_BASE 0x80000
#define KERN_STACK_SIZE 0x10000

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

// Forward declarations
void kinit(void);
void kinit2(void);
void kmain(void);
void kmain2(void);
void switch_to_el2(void);
void switch_to_el1(void);
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
static inline void eret(void) {
    __asm__ volatile("eret");
}

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

void switch_to_el2(void) {
    if (current_el() == 3) {
        uint64_t sp = READ_SYSTEM_REG(SP_EL0);
        WRITE_SYSTEM_REG(SP_EL2, sp);
        uint64_t scr = (1ULL << 10) | (1ULL << 8) | (1ULL << 7) | (1ULL << 6) | (1ULL << 0);
        WRITE_SYSTEM_REG(SCR_EL3, scr);
        uint64_t spsr = 0x9 | (1ULL << 6) | (1ULL << 7) | (1ULL << 8) | (1ULL << 9);
        WRITE_SYSTEM_REG(SPSR_EL3, spsr);
        WRITE_SYSTEM_REG(ELR_EL3, (uint64_t)switch_to_el2);
        eret();
    }
}

void switch_to_el1(void) {
    if (current_el() == 2) {
        uint64_t sp = READ_SYSTEM_REG(SP_EL0);
        WRITE_SYSTEM_REG(SP_EL1, sp);
        uint64_t cnthctl = READ_SYSTEM_REG(CNTHCTL_EL2);
        WRITE_SYSTEM_REG(CNTHCTL_EL2, cnthctl | (1ULL << 1) | (1ULL << 0));
        WRITE_SYSTEM_REG(CNTVOFF_EL2, 0);
        WRITE_SYSTEM_REG(HCR_EL2, (1ULL << 31) | (1ULL << 29));
        WRITE_SYSTEM_REG(CPTR_EL2, 0);
        uint64_t cpacr = READ_SYSTEM_REG(CPACR_EL1);
        WRITE_SYSTEM_REG(CPACR_EL1, cpacr | (3ULL << 20));
        WRITE_SYSTEM_REG(SCTLR_EL1, 0x30C50830);
        WRITE_SYSTEM_REG(VBAR_EL1, (uint64_t)&vectors_start);
        uint64_t spsr = 0x5 | (1ULL << 6) | (1ULL << 7) | (1ULL << 8) | (1ULL << 9);
        WRITE_SYSTEM_REG(SPSR_EL2, spsr);
        WRITE_SYSTEM_REG(ELR_EL2, (uint64_t)switch_to_el1);
        eret();
    }
}

// Per-core initialization status
volatile uint32_t cores_ready = 0;

// init.h
void start2(void) __attribute__((noreturn));

void __attribute__((noreturn)) start2(void) {
    // Ensure we're in the correct exception level
    if (current_el() > 1) {
        switch_to_el1();
    }
    
    // Initialize core-specific hardware
    uint64_t core_id = READ_SYSTEM_REG(MPIDR_EL1) & 0xFF;
    uint64_t stack_top = KERN_STACK_BASE - (core_id * KERN_STACK_SIZE);
    
    // Set up stack and vector table
    WRITE_SYSTEM_REG(SP_EL1, stack_top);
    WRITE_SYSTEM_REG(VBAR_EL1, (uint64_t)&vectors_start);
    
    // Signal ready
    dmb();
    cores_ready |= (1 << core_id);
    dsb();
    sev();
    
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

void initialize_app_cores(void) {
    printf("Initializing secondary cores...\n");

    // Ensure QEMU spin table address is accessible
    volatile uint64_t* qemu_spin_table = (volatile uint64_t*)QEMU_SPIN_TABLE;

    // Set entry point for each secondary core
    for (int core = 1; core < NCORES; core++) {
        printf("Setting entry for core %d\n", core);
        qemu_spin_table[core] = (uint64_t)start2;
    }

    // Ensure the writes complete before signaling cores
    dsb();
    sev();  // Wake up secondary cores
}

void kmain2(void) {
    printf("Hello from core %lx!\n", READ_SYSTEM_REG(MPIDR_EL1) & 0xFF);
    uint64_t core_num = READ_SYSTEM_REG(MPIDR_EL1) & 0xFF;
    SPINNING_BASE[core_num] = 0;
    while(1) { }
}

void set_vector_table(void) {
    extern uint64_t vectors_start;
    WRITE_SYSTEM_REG(VBAR_EL1, (uint64_t)&vectors_start);
    isb();  // Instruction synchronization barrier
}

void __attribute__((noreturn)) kinit(void) {
    // Initialize UART first for debug output
    uart_init();
    uart_puts("Starting initialization...\n");
    
    // Get current exception level
    uint64_t el = current_el();
    printf("Current EL: %d\n", el);
    
    // Initialize stack for current core
    uint64_t mpidr = READ_SYSTEM_REG(MPIDR_EL1);
    uint64_t core_id = mpidr & 0xFF;
    uint64_t stack_top = KERN_STACK_BASE - (core_id * KERN_STACK_SIZE);
    WRITE_SYSTEM_REG(SP_EL0, stack_top);
    
    // Switch exception levels if needed
    if (el > 2) {
        uart_puts("Switching to EL2\n");
        switch_to_el2();
    }
    if (el > 1) {
        uart_puts("Switching to EL1\n");
        switch_to_el1();
    }
    
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