#pragma once
// #include <stdint.h>
// #include <stddef.h>
// #include <stdio.h>
#include "stddef.h"


//extern void heap_init();
extern void heapInit(void* start, int bytes);
extern void* malloc(size_t size);
extern void free(void* p);