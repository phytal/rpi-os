SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
ASM_SRCS = $(wildcard *.S)
ASM_OBJS = $(ASM_SRCS:.S=.o)

CFLAGS = -Wall -O2 -ffreestanding -nostdinc -nostdlib -nostartfiles
LDFLAGS = -L./lib
LIBS = -lsd


all: clean kernel8.img

%.o: %.S
	aarch64-elf-gcc $(CFLAGS) -c $< -o $@

%.o: %.c
	aarch64-elf-gcc $(CFLAGS) -c $< -o $@

lib/libsd.a: 
	ranlib lib/libsd.a

kernel8.elf: $(ASM_OBJS) $(OBJS) lib/libsd.a
	aarch64-elf-ld -nostdlib $(ASM_OBJS) $(OBJS) -T linker.ld -o kernel8.elf $(LDFLAGS) $(LIBS)

kernel8.img: kernel8.elf
	aarch64-elf-objcopy -O binary kernel8.elf kernel8.img

clean:
	rm -f *.o kernel8.elf kernel8.img >/dev/null 2>/dev/null || true

run:
	qemu-system-aarch64 -M raspi3b -kernel kernel8.img -drive file=disk-images/bn.img,if=sd,format=raw -serial stdio
