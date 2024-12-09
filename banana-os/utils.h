// .global get_el
#pragma once

#ifndef UTILS_H
#define UTILS_H

#include "stdint.h"

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

void *memcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char*) dest;
    const unsigned char *s = (const unsigned char*) src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}


#endif // UTILS_H