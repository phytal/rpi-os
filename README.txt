santa-os

A bare-metal kernel for the Raspberry Pi 3 (BCM2837, Cortex-A53), written in
C and AArch64 assembly. It boots all four cores to EL1, mounts an ext2
filesystem from the SD card and prints the files in its root directory, and
handles a 1 Hz timer interrupt on core 0.

What it does
- Boot: each core sets up its own stack, drops from EL2 to EL1 and installs
  the exception vector table (santa-os/boot.S). Cores 1-3 are released one at
  a time through the firmware's spin table at 0xd8.
- Interrupts: BCM2837 interrupt controller + system timer (C1 compare, 1 Hz).
- Heap: first-fit allocator with coalescing, 16-byte aligned (heap.c).
- Storage: EMMC/SD driver (sd.c) and read-only ext2 (ext2.c).
- Console: PL011 UART at 115200 8N1, printf.

Not there yet: MMU and caches (both off), locking, processes/scheduler, user
mode and system calls, UART input, filesystem writes.

Layout
  santa-os/              the kernel
    boot.S               entry point, per-core setup (stack, EL1, VBAR)
    vectors.S            exception vector table, IRQ entry/exit
    init.c               kinit(): heap, filesystem, interrupts; core start-up
    main.c               kmain(): starts cores 1-3, then idles
    heap.c ext2.c sd.c   heap, filesystem, SD card
    uart.c sprintf.c     console output
    irq.c gic.c timer.c  interrupt controller and system timer
    linker.ld            links the image at 0x80000
    disk-images/bn.img   16 MiB ext2 image used by `make run`
    text-files/          the files stored in bn.img
  mnt/                   Pi firmware + config.txt for a real SD card boot
                         partition (copy santa-os/kernel8.img next to them)

Build and run (needs aarch64-elf-gcc and qemu-system-aarch64)
  cd santa-os
  make all
  make run          # QEMU raspi3b, 4 cores, bn.img as the SD card
                    # stop QEMU with Ctrl-C

Debugging tip: add `-d int,guest_errors` to the qemu command in the Makefile
to log every exception (with ESR/ELR/FAR) taken by every core.

Real hardware: copy mnt/* and santa-os/kernel8.img to the FAT boot partition
of an SD card. Note the ext2 code expects the filesystem at the very start of
the card (no partition table), which is how bn.img is laid out for QEMU; on a
real card it would need to read the partition table first.

History
The team's work ended in two kernels: banana-os (SD card, ext2, heap; one core
at EL2) and santa-os (multicore attempt, forked before banana-os's cleanup).
They were merged into santa-os in Nov 2026, and the multicore boot, interrupt
return path and linker layout were fixed. banana-os, the CS3210 course
material (lib/, user/, ext/, bin/), prebuilt images and earlier experiments
were removed at the same time; they are all still in git history before that
commit.

Credits
- delays.c and sprintf.c come from bzt's raspi3-tutorial (MIT), and sd.c,
  parts of uart.c and linker.ld are based on it:
  https://github.com/bztsrc/raspi3-tutorial
- heap.c is adapted from the first-fit heap in Dr. Gheith's OS course at UT.
