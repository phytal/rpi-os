// constants.h
#ifndef CONSTANTS_H
#define CONSTANTS_H

#define KERN_STACK_BASE 0x80000
#define KERN_STACK_SIZE 0x10000
#define NCORES 4

// Heap: 1 MiB - 6 MiB, above the kernel image and the per-core stacks
// (which sit below 0x80000).
#define HEAP_START 0x100000
#define HEAP_SIZE  (5 * 1024 * 1024)

#endif
