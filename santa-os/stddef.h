#ifndef STDDEF_H
#define STDDEF_H

// Fixed-width types live in stdint.h; this file only adds size_t and NULL.
// (It used to redefine uint64_t/int8_t with different types than stdint.h,
// so any file including both failed to compile.)
#include "stdint.h"

typedef unsigned long size_t;

#define NULL ((void*)0)

#endif /* STDDEF_H */
