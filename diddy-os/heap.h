#pragma once
// #include <stdint.h>
// #include <stddef.h>
// #include <stdio.h>

typedef unsigned int size_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned int uintptr_t;

//extern void heap_init();
extern void heapInit(void* start, int bytes);
extern void* malloc(size_t size);
extern void free(void* p);