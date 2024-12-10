Banana OS Documentation

Project Overview

A bare metal OS for Raspberry Pi 3B+ written in C with:

- Custom bootloader
- EXT2 filesystem support
- SD card driver
- UART serial communication

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
