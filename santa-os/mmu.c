#include "stdint.h"
#include "uart.h"

#define MM_TYPE_PAGE_TABLE    0x3
#define MM_TYPE_PAGE         0x3
#define MM_TYPE_BLOCK        0x1
#define MM_ACCESS            (1 << 10)
#define MM_KERNEL_RW         ((1 << 6) | (1 << 7))   // AP[2:1] = 0b00, RW at EL1
#define MM_NORMAL           (0x4 << 2)              // Normal memory
#define MM_DEVICE           (0x0 << 2)              // Device memory

#define MAIR_DEVICE_nGnRnE   0x00
#define MAIR_NORMAL_NC       0x44
#define MAIR_VALUE          ((MAIR_DEVICE_nGnRnE << 0) | (MAIR_NORMAL_NC << 8))

static uint64_t mmu_table[512] __attribute__((aligned(4096)));

void mmu_init(void) {
    uart_puts("Initializing MMU...\n");

    // Clear table first
    for (int i = 0; i < 512; i++) {
        mmu_table[i] = 0;
    }

    // Map first 1GB (Device memory)
    mmu_table[0] = (0x00000000ULL) | 
                   MM_DEVICE | 
                   MM_ACCESS | 
                   MM_KERNEL_RW |
                   MM_TYPE_BLOCK;

    // Map second 1GB (Normal memory)
    mmu_table[1] = (0x40000000ULL) | 
                   MM_NORMAL |
                   MM_ACCESS |
                   MM_KERNEL_RW |
                   MM_TYPE_BLOCK;

    // Set up translation control register
    uint64_t tcr = (16ULL << 0) |  // T0SZ = 16 (48-bit address space)
                   (0ULL << 6) |   // Inner Non-cacheable
                   (0ULL << 8) |   // Outer Non-cacheable
                   (3ULL << 10) |  // Inner shareable
                   (1ULL << 14) |  // TG0 = 4KB
                   (16ULL << 16);  // T1SZ = 16

    // Set up memory attribute indirection register
    __asm__ volatile ("msr mair_el1, %0" :: "r"(MAIR_VALUE));
    __asm__ volatile ("isb");

    // Set up translation table base register
    __asm__ volatile ("msr ttbr0_el1, %0" :: "r"(mmu_table));
    __asm__ volatile ("isb");

    // Set up translation control register
    __asm__ volatile ("msr tcr_el1, %0" :: "r"(tcr));
    __asm__ volatile ("isb");

    // Enable MMU
    uint64_t sctlr;
    __asm__ volatile ("mrs %0, sctlr_el1" : "=r"(sctlr));
    sctlr |= (1 << 0);    // M bit - Enable MMU
    sctlr |= (1 << 2);    // C bit - Enable data cache
    sctlr |= (1 << 12);   // I bit - Enable instruction cache
    __asm__ volatile ("msr sctlr_el1, %0" :: "r"(sctlr));
    __asm__ volatile ("isb");

    uart_puts("MMU initialized successfully\n");
}