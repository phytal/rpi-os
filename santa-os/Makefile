SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
ASM_SRCS = boot.S vectors.S
ASM_OBJS = $(ASM_SRCS:.S=.o)
# -mgeneral-regs-only: the IRQ path doesn't save FP/SIMD registers.
# -mstrict-align: with the MMU off all memory is Device memory, where
#   unaligned accesses fault.
CFLAGS = -Wall -O2 -ffreestanding -nostdinc -nostdlib -nostartfiles -mgeneral-regs-only -mstrict-align -I.
DISK_IMG = disk-images/bn.img

all: clean kernel8.img

%.o: %.S
	aarch64-elf-gcc $(CFLAGS) -c $< -o $@

%.o: %.c
	aarch64-elf-gcc $(CFLAGS) -c $< -o $@

kernel8.elf: $(ASM_OBJS) $(OBJS)
	aarch64-elf-ld -nostdlib $(ASM_OBJS) $(OBJS) -T linker.ld -o kernel8.elf

kernel8.img: kernel8.elf
	aarch64-elf-objcopy -O binary kernel8.elf kernel8.img

clean:
	rm -f *.o kernel8.elf kernel8.img >/dev/null 2>/dev/null || true

run:
	qemu-system-aarch64 -M raspi3b -smp 4 -kernel kernel8.img -serial stdio \
		-drive file=$(DISK_IMG),if=sd,format=raw
