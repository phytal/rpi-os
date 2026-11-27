#include "utils.h"

// Without no-tree-loop-distribute-patterns, GCC at -O2 can recognise these
// loops as memcpy/memset and compile them into calls to themselves.
#define NO_LIBCALL __attribute__((optimize("no-tree-loop-distribute-patterns")))

NO_LIBCALL void *memcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char*) dest;
    const unsigned char *s = (const unsigned char*) src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}

NO_LIBCALL void *memset(void *dest, int c, size_t n) {
    unsigned char *d = (unsigned char*) dest;
    while (n--) {
        *d++ = (unsigned char) c;
    }
    return dest;
}
