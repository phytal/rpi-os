ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET := target/aarch64-unknown-none/release/${KERN}
OBJCPY := cargo objcopy --bin $(KERN) --release -- -O binary --strip-all
TTY_PATH := /dev/tty.SLAB_USBtoUART
SDCARD ?= $(ROOT)/jellyos/user/fs.img

.PHONY: all build qemu transmit objdump nm check clean install test

all: build

build:
	@echo "+ Building build/$(KERN).elf [build/$@]"
	@cargo build --release
	@mkdir -p build
	@cp -f $(TARGET) build/$(KERN).elf

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@$(OBJCPY) build/$(KERN).bin

build-all:
	@(cd ../ext/uspi/lib; make)
	cp -f ../ext/uspi/lib/libuspi.a ./.cargo/
	@(cd ../ext/uspi/env/lib; make)
	cp -f ../ext/uspi/env/lib/libuspienv.a ./.cargo/
	touch src/main.rs
	make build

check:
	@cargo check

qemu: build
	@# ./qemu.sh build/$(KERN).bin
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd $(QEMU_ARGS)

qemu-gdb: build
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd -s -S

qemu-asm: build
	./qemu.sh build/$(KERN).bin -drive file=$(SDCARD),format=raw,if=sd -d in_asm

transmit: build
	@echo "+ Transmitting build/$(KERN).bin to $(TTY_PATH)"
	ttywrite -i build/$(KERN).bin $(TTY_PATH)
	screen $(TTY_PATH) 115200

run: build
	./run.sh

objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex build/$(KERN).elf

nm: build
	cargo nm build/$(KERN).elf

clean:
	cargo clean
	rm -rf build

test:
	cargo test --target=$(shell $(ROOT)/bin/get-host-target.sh)
