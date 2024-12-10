#include "stdint.h"
#include "sprintf.h"
#include "uart.h"

// Forward declarations for external modules
void init_logger(void);
void initialize_app_cores(void);
void spin_sleep(uint64_t milliseconds);

// // Structure definitions
// typedef struct {
//     // Add necessary fields
//     bool initialized;
// } Allocator;

// typedef struct {
//     bool initialized;
// } FileSystem;

// typedef struct {
//     bool initialized;
// } GlobalScheduler;

// typedef struct {
//     bool initialized;
// } VMManager;

// typedef struct {
//     bool initialized;
// } Usb;

// typedef struct {
//     bool initialized;
// } GlobalIrq;

// typedef struct {
//     bool initialized;
// } LocalIrq;

// typedef struct {
//     bool initialized;
// } Fiq;

// typedef struct {
//     bool initialized;
// } GlobalEthernetDriver;

// Global variables
// extern Allocator ALLOCATOR;
// extern FileSystem FILESYSTEM;
// extern GlobalScheduler SCHEDULER;
// extern VMManager VMM;
// extern Usb USB;
// extern GlobalIrq GLOBAL_IRQ;
// extern LocalIrq IRQ;
// extern Fiq FIQ;
// extern GlobalEthernetDriver ETHERNET;

// External symbols from linker script
extern uint64_t __text_beg;
extern uint64_t __text_end;
extern uint64_t __bss_beg;
extern uint64_t __bss_end;

// // Initialization functions
// void init_allocator(Allocator* alloc) {
//     alloc->initialized = true;
//     // Add initialization logic
// }

// void init_filesystem(FileSystem* fs) {
//     fs->initialized = true;
//     // Add initialization logic
// }

// void init_vmm(VMManager* vmm) {
//     vmm->initialized = true;
//     // Add initialization logic
// }

// void init_scheduler(GlobalScheduler* sched) {
//     sched->initialized = true;
//     // Add initialization logic
// }

// Logging function
void log_layout(void) {
    // init_logger();

    // Info logging
    // // Note: Replace with your logging implementation
    // kprintln("text beg: %016llx, end: %016llx",
    //          (uint64_t)&__text_beg, (uint64_t)&__text_end);
    // kprintln("bss  beg: %016llx, end: %016llx",
    //          (uint64_t)&__bss_beg, (uint64_t)&__bss_end);
}

// Per-core initialization
void __attribute__((noreturn)) per_core_main(void) {
    // Wait for VMM initialization
    // while (!VMM.initialized) { }

    // // Start scheduler
    // SCHEDULER.initialized = true;  // Additional scheduler startup logic needed

    // Should never reach here
    while(1) {
        // printf("Core running...\n");
     }
}

// Main kernel entry point for bootstrap core
void __attribute__((noreturn)) kmain(void) {
    // Initial delay for TTY stability
    // spin_sleep(500);  // 500ms delay
    
    // log_layout();

    // // Initialize core systems
    // init_allocator(&ALLOCATOR);
    // init_filesystem(&FILESYSTEM);
    // init_vmm(&VMM);
    // init_scheduler(&SCHEDULER);

    // Initialize additional cores
    initialize_app_cores();

    per_core_main();

    // Should never reach here
    while(1) { }
}



// Global variable definitions
// Allocator ALLOCATOR = { .initialized = false };
// FileSystem FILESYSTEM = { .initialized = false };
// GlobalScheduler SCHEDULER = { .initialized = false };
// VMManager VMM = { .initialized = false };
// Usb USB = { .initialized = false };
// GlobalIrq GLOBAL_IRQ = { .initialized = false };
// LocalIrq IRQ = { .initialized = false };
// Fiq FIQ = { .initialized = false };
// GlobalEthernetDriver ETHERNET = { .initialized = false };