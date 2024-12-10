# boot.S
#include "constants.h"

// Must be at start of binary
.section ".text.boot"
.global _start

// Entry point for all cores
.org 0x0
_start:
    mrs x0, MPIDR_EL1
    and x0, x0, #0xFF       // Get core ID

    // Check if primary core (core 0)
    cbz x0, primary_boot

secondary_boot:
    ldr x1, =0xD8           // Address of spin table
    lsl x2, x0, #3          // Multiply core ID by 8 (size of uint64_t)
spin_wait:
    wfe                     // Wait for event
    ldr x3, [x1, x2]        // Load entry from spin table
    cbz x3, spin_wait       // If zero, continue waiting
    br x3                   // Jump to start2

.org 0x100
primary_boot:
    // Set up initial stack
    ldr x1, =KERN_STACK_BASE
    mov sp, x1
    
    // Clear BSS
    bl zeros_bss
    
    // Initialize UART 
    bl uart_init
    
    // Continue boot
    bl kinit
    
    // Should never reach here
    b .
