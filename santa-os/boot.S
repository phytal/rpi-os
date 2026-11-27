// boot.S
#include "constants.h"

.section ".text.boot"

// Core 0 enters here. On QEMU and current Pi firmware, cores 1-3 are parked
// by the armstub at 0x0 and only leave when core 0 writes an address into the
// spin table at 0xd8 + 8*core (see initialize_app_cores).
.global _start
_start:
    mrs     x0, mpidr_el1
    and     x0, x0, #3
    cbz     x0, 2f

    // Old firmware (kernel_old=1) releases every core here; park the extra
    // ones on the same spin table the stock armstub uses.
    mov     x1, #0xd8
1:  wfe
    ldr     x2, [x1, x0, lsl #3]
    cbz     x2, 1b
    br      x2

2:  bl      core_setup          // stack + drop to EL1
    bl      zeros_bss
    bl      uart_init
    bl      kinit
    b       halt

// Address handed to cores 1-3 through the spin table. They arrive at EL2
// with no stack, so nothing written in C may run before core_setup.
.global secondary_start
secondary_start:
    bl      core_setup
    bl      start2
halt:
    wfe
    b       halt

// Per-core setup: pick this core's stack, leave EL3/EL2 for EL1h, install
// the EL1 vector table. Returns (via x30) running at EL1 on the new stack.
core_setup:
    mrs     x0, mpidr_el1
    and     x0, x0, #3
    mov     x1, #KERN_STACK_SIZE
    mul     x1, x1, x0
    ldr     x2, =KERN_STACK_BASE
    sub     x2, x2, x1          // x2 = stack top for this core

    mrs     x0, CurrentEL
    lsr     x0, x0, #2
    cmp     x0, #3
    b.ne    3f
    // EL3 -> EL2 (only if something booted us at EL3)
    mov     x0, #0x5b1          // NS, SMD, HCE, RW, RES1
    msr     scr_el3, x0
    mov     x0, #0x3c9          // EL2h, DAIF masked
    msr     spsr_el3, x0
    adr     x0, 3f
    msr     elr_el3, x0
    eret

3:  mrs     x0, CurrentEL
    lsr     x0, x0, #2
    cmp     x0, #2
    b.ne    4f
    // EL2 -> EL1
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3          // EL1 may use the physical counter/timer
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr
    mov     x0, #0x33ff         // don't trap FP/SIMD to EL2
    msr     cptr_el2, x0
    msr     hstr_el2, xzr
    mov     x0, #(1 << 31)      // EL1 is AArch64
    msr     hcr_el2, x0
    ldr     x0, =0x30C50830     // MMU and caches off
    msr     sctlr_el1, x0
    mov     x0, #(3 << 20)      // FP/SIMD allowed at EL1
    msr     cpacr_el1, x0
    msr     sp_el1, x2
    mov     x0, #0x3c5          // EL1h, DAIF masked
    msr     spsr_el2, x0
    adr     x0, 5f
    msr     elr_el2, x0
    eret

4:  mov     sp, x2              // already at EL1
5:  ldr     x0, =vectors_start
    msr     vbar_el1, x0
    isb
    ret
