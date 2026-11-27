.section ".text.vectors"

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

.global vec_context_switch
vec_context_switch:
    
    mov x28, lr
    
    bl vec_context_save    

    mov x0, x29 // info
    mrs x1, ESR_EL1 // esr
    mov x2, SP // tf

    bl handle_exception

    mov lr, x28
    b vec_context_restore // teleport


.global vec_context_save
vec_context_save:
    // Save general-purpose registers
    stp x26, x27, [SP, #-16]!
    stp x24, x25, [SP, #-16]!
    stp x22, x23, [SP, #-16]!
    stp x20, x21, [SP, #-16]!
    stp x18, x19, [SP, #-16]!
    stp x16, x17, [SP, #-16]!
    stp x14, x15, [SP, #-16]!
    stp x12, x13, [SP, #-16]!
    stp x10, x11, [SP, #-16]!
    stp x8, x9, [SP, #-16]!
    stp x6, x7, [SP, #-16]!
    stp x4, x5, [SP, #-16]!
    stp x2, x3, [SP, #-16]!
    stp x0, x1, [SP, #-16]!

    // Save SIMD and FP registers
    stp q30, q31, [SP, #-32]!
    stp q28, q29, [SP, #-32]!
    stp q26, q27, [SP, #-32]!
    stp q24, q25, [SP, #-32]!
    stp q22, q23, [SP, #-32]!
    stp q20, q21, [SP, #-32]!
    stp q18, q19, [SP, #-32]!
    stp q16, q17, [SP, #-32]!
    stp q14, q15, [SP, #-32]!
    stp q12, q13, [SP, #-32]!
    stp q10, q11, [SP, #-32]!
    stp q8, q9, [SP, #-32]!
    stp q6, q7, [SP, #-32]!
    stp q4, q5, [SP, #-32]!
    stp q2, q3, [SP, #-32]!
    stp q0, q1, [SP, #-32]!

    // Save system registers
    mrs x0, TTBR0_EL1
    mrs x1, TTBR1_EL1
    stp x0, x1, [SP, #-16]!

    mrs x0, SP_EL0
    mrs x1, TPIDR_EL0
    stp x0, x1, [SP, #-16]!

    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1, [SP, #-16]!

    ret

.global vec_context_restore
vec_context_restore:
    // Restore system registers
    ldp x0, x1, [SP], #16
    msr ELR_EL1, x0
    msr SPSR_EL1, x1

    ldp x0, x1, [SP], #16
    msr SP_EL0, x0
    msr TPIDR_EL0, x1

    ldp x0, x1, [SP], #16
    msr TTBR0_EL1, x0
    msr TTBR1_EL1, x1

    dsb ishst
    tlbi vmalle1
    dsb ish
    isb

    // Restore SIMD and FP registers
    ldp q0, q1, [SP], #32
    ldp q2, q3, [SP], #32
    ldp q4, q5, [SP], #32
    ldp q6, q7, [SP], #32
    ldp q8, q9, [SP], #32
    ldp q10, q11, [SP], #32
    ldp q12, q13, [SP], #32
    ldp q14, q15, [SP], #32
    ldp q16, q17, [SP], #32
    ldp q18, q19, [SP], #32
    ldp q20, q21, [SP], #32
    ldp q22, q23, [SP], #32
    ldp q24, q25, [SP], #32
    ldp q26, q27, [SP], #32
    ldp q28, q29, [SP], #32
    ldp q30, q31, [SP], #32

    // Restore general-purpose registers
    ldp x0, x1, [SP], #16
    ldp x2, x3, [SP], #16
    ldp x4, x5, [SP], #16
    ldp x6, x7, [SP], #16
    ldp x8, x9, [SP], #16
    ldp x10, x11, [SP], #16
    ldp x12, x13, [SP], #16
    ldp x14, x15, [SP], #16
    ldp x16, x17, [SP], #16
    ldp x18, x19, [SP], #16
    ldp x20, x21, [SP], #16
    ldp x22, x23, [SP], #16
    ldp x24, x25, [SP], #16
    ldp x26, x27, [SP], #16

    ret

// pair instructions ensure stack alignment of 16
.macro HANDLER source, kind
    .align 7
    stp     lr, xzr, [SP, #-16]!
    stp     x28, x29, [SP, #-16]!
    
    mov     x29, \source
    movk    x29, \kind, LSL #16
    bl      vec_context_switch
    
    ldp     x28, x29, [SP], #16
    ldp     lr, xzr, [SP], #16
    eret
.endm
    
.align 11

.global vectors
vectors:
    HANDLER 0, 0
    HANDLER 0, 1
    HANDLER 0, 2
    HANDLER 0, 3
    HANDLER 1, 0
    HANDLER 1, 1
    HANDLER 1, 2
    HANDLER 1, 3
    HANDLER 2, 0
    HANDLER 2, 1
    HANDLER 2, 2
    HANDLER 2, 3
    HANDLER 3, 0
    HANDLER 3, 1
    HANDLER 3, 2
    HANDLER 3, 3
