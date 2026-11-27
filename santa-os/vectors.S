.section ".text.vectors", "ax"
.align 11
.globl vectors_start
vectors_start:
    // Current EL with SP0
    .align 7
    b exception_handler    // Synchronous Exception
    .align 7
    b irq_entry            // IRQ
    .align 7
    b exception_handler    // FIQ
    .align 7
    b exception_handler    // SError

    // Current EL with SPx
    .align 7
    b exception_handler
    .align 7
    b irq_entry            // IRQ
    .align 7
    b exception_handler    // FIQ
    .align 7
    b exception_handler    // SError

    // Lower EL using AArch64
    .align 7
    b exception_handler
    .align 7
    b irq_entry
    .align 7
    b exception_handler
    .align 7
    b exception_handler

    // Lower EL using AArch32
    .align 7
    b exception_handler
    .align 7
    b irq_entry
    .align 7
    b exception_handler
    .align 7
    b exception_handler

// Save x0-x30 plus ELR/SPSR, then restore and eret. The previous version did
// "mov x28, lr ... ret", which clobbered x28/x30 of the interrupted code and
// returned with ret instead of eret, so the first IRQ never came back.
.macro KERNEL_ENTRY
    sub     sp, sp, #16 * 17
    stp     x0, x1,   [sp, #16 * 0]
    stp     x2, x3,   [sp, #16 * 1]
    stp     x4, x5,   [sp, #16 * 2]
    stp     x6, x7,   [sp, #16 * 3]
    stp     x8, x9,   [sp, #16 * 4]
    stp     x10, x11, [sp, #16 * 5]
    stp     x12, x13, [sp, #16 * 6]
    stp     x14, x15, [sp, #16 * 7]
    stp     x16, x17, [sp, #16 * 8]
    stp     x18, x19, [sp, #16 * 9]
    stp     x20, x21, [sp, #16 * 10]
    stp     x22, x23, [sp, #16 * 11]
    stp     x24, x25, [sp, #16 * 12]
    stp     x26, x27, [sp, #16 * 13]
    stp     x28, x29, [sp, #16 * 14]
    mrs     x21, elr_el1
    mrs     x22, spsr_el1
    stp     x30, x21, [sp, #16 * 15]
    str     x22,      [sp, #16 * 16]
.endm

.macro KERNEL_EXIT
    ldr     x22,      [sp, #16 * 16]
    ldp     x30, x21, [sp, #16 * 15]
    msr     elr_el1, x21
    msr     spsr_el1, x22
    ldp     x0, x1,   [sp, #16 * 0]
    ldp     x2, x3,   [sp, #16 * 1]
    ldp     x4, x5,   [sp, #16 * 2]
    ldp     x6, x7,   [sp, #16 * 3]
    ldp     x8, x9,   [sp, #16 * 4]
    ldp     x10, x11, [sp, #16 * 5]
    ldp     x12, x13, [sp, #16 * 6]
    ldp     x14, x15, [sp, #16 * 7]
    ldp     x16, x17, [sp, #16 * 8]
    ldp     x18, x19, [sp, #16 * 9]
    ldp     x20, x21, [sp, #16 * 10]
    ldp     x22, x23, [sp, #16 * 11]
    ldp     x24, x25, [sp, #16 * 12]
    ldp     x26, x27, [sp, #16 * 13]
    ldp     x28, x29, [sp, #16 * 14]
    add     sp, sp, #16 * 17
    eret
.endm

.global irq_entry
irq_entry:
    KERNEL_ENTRY
    bl      handle_irq
    KERNEL_EXIT

.global exception_handler
exception_handler:
    KERNEL_ENTRY
    mrs     x0, esr_el1
    mrs     x1, elr_el1
    mrs     x2, spsr_el1
    bl      handle_exception    // prints and halts
    KERNEL_EXIT
