// .global get_el
// get_el:
//     mrs x0, CurrentEL
//     lsr x0, x0, #2
//     ret

// .global put32
// put32:
//     str w1, [x0]
//     ret
// .global get32
// get32:
//     ldr w0, [x0]
//     ret

// .global delay
// delay:
//     subs x0, x0, #1
//     bne delay
//     ret


static inline unsigned int get_el() {
    printf("what is happening\n");
    // asm volatile("mrs x0, CurrentEL");
    // asm volatile("lsr x0, x0, #2");
    // asm volatile("ret");
    unsigned int el;
    asm volatile (
        "mrs %0, CurrentEL\n"  // Read CurrentEL into the C variable 'el'
        "lsr %0, %0, #2"       // Logical shift right by 2 to extract EL
        : "=r" (el)            // Output operand
        :
        : "memory"             // Indicate that memory might be affected
    );
    return el;
}