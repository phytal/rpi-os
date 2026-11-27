#ifndef UTILS_H
#define UTILS_H

#include "stddef.h"

static inline unsigned int get_el(void) {
    unsigned int el;
    asm volatile (
        "mrs %0, CurrentEL\n"  // Read CurrentEL into the C variable 'el'
        "lsr %0, %0, #2"       // Logical shift right by 2 to extract EL
        : "=r" (el)
        :
        : "memory"
    );
    return el;
}

// Defined in utils.c. GCC may also emit calls to these on its own, even with
// -ffreestanding, so they have to exist somewhere in the kernel.
void *memcpy(void *dest, const void *src, size_t n);
void *memset(void *dest, int c, size_t n);

#endif // UTILS_H
