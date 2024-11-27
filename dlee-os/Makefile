CFLAGS = -Wall -O2 -ffreestanding -nostdinc -nostdlib -nostartfiles -Wextra
LDFLAGS = -T linker.ld

all: clean kernel8.img

boot.o: boot.S
	aarch64-elf-as -c boot.S -o boot.o

kernel.o: kernel.c
	aarch64-elf-gcc $(CFLAGS) -c kernel.c -o kernel.o

kernel8.elf: boot.o kernel.o
	aarch64-elf-gcc $(LDFLAGS) -o kernel8.elf -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc

kernel8.img: kernel8.elf
	aarch64-elf-objcopy kernel8.elf -O binary kernel8.img

clean:
	rm -f *.o kernel8.elf kernel8.img

run: all
	qemu-system-aarch64 -M raspi3b -serial stdio -kernel kernel8.img