// boot.S
.section ".text.boot"

.global _start

_start:
    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

    // Configure EL2 for interrupt handling
    mrs     x0, hcr_el2
    orr     x0, x0, #(1 << 3)     // FMO: FIQ routing to EL2
    orr     x0, x0, #(1 << 4)     // IMO: IRQ routing to EL2
    orr     x0, x0, #(1 << 5)     // AMO: Async abort routing to EL2
    msr     hcr_el2, x0

    // Set up stack pointer for EL2
    ldr     x1, =_start
    mov     sp, x1

    // Enable interrupt unmasking
    msr     daifclr, #2

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
3:  cbz     w2, 4f
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, 3b

    // jump to C code, should not return
4:  bl      kernel_main
    // for failsafe, halt this core too
    b       1b