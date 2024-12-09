Banana OS Documentation

Project Overview

A bare metal OS for Raspberry Pi 3B+ written in C with:

- Custom bootloader
- EXT2 filesystem support
- SD card driver
- UART serial communication

Project Structure
.
├── Makefile
├── README
├── boot.S
├── boot.o
├── delays.c
├── delays.h
├── delays.o
├── disk-images
│   ├── bn.img
│   └── glonk.dd
├── ext2.c
├── ext2.h
├── ext2.o
├── gic.c
├── gic.h
├── gic.o
├── gpio.h
├── heap.c
├── heap.h
├── heap.o
├── irq.c
├── irq.h
├── irq.o
├── kernel.c
├── kernel.o
├── lib
│   └── libsd.a
├── linker.ld
├── mbox.c
├── mbox.h
├── mbox.o
├── mnt
├── sd.c
├── sd.h
├── sd.o
├── sprintf.c
├── sprintf.h
├── sprintf.o
├── stdint.h
├── text-files
│   ├── grr.txt
│   ├── skib.txt
│   └── toil.txt
├── timer.c
├── timer.h
├── timer.o
├── uart.c
├── uart.h
├── uart.o
├── utils.S
├── utils.h
├── utils.o
├── vectors.S
└── vectors.o

Build Process

1. Cross-compile for AArch64
2. Link with custom linker script
3. Generate raw binary image
4. Create SD card image with EXT2 filesystem

make all       
make run      # Run in QEMU

Testing

QEMU emulation with serial output
Physical hardware testing
Unit tests for filesystem code
