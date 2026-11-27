#pragma once
#include "stddef.h"


extern void heapInit(void* start, int bytes);
extern void* malloc(size_t size);
extern void free(void* p);