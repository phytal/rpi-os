#include "stdint.h"

void initialize_app_cores(void);

// Core 0's idle loop once everything is up; timer interrupts arrive here.
void __attribute__((noreturn)) per_core_main(void) {
    while(1) {
    }
}

// Called at the end of kinit() on core 0.
void __attribute__((noreturn)) kmain(void) {
    initialize_app_cores();
    per_core_main();
}
