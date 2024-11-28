ROOT := $(shell git rev-parse --show-toplevel)

KERN := boot
TARGET := target/aarch64-unknown-none/release/${KERN}
OBJCPY := cargo objcopy --bin $(KERN) --release -- -O binary --strip-all
SDCARD ?= $(ROOT)/user/fs.img

.PHONY: all build qemu objdump nm check clean install test

all: build

build:
	@echo "+ Building build/$(KERN).elf [build/$@]"
	@cargo build --release
	@mkdir -p build
	@cp -f $(TARGET) build/$(KERN).elf

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@$(OBJCPY) build/$(KERN).bin

check:
	@cargo check

qemu: build
	./qemu.sh build/$(KERN).elf -drive file=$(SDCARD),format=raw,if=sd $(QEMU_ARGS)

qemu-gdb: build
	./qemu.sh build/$(KERN).elf -drive file=$(SDCARD),format=raw,if=sd -s -S


objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex build/$(KERN).elf

nm: build
	cargo nm build/$(KERN).elf

clean:
	cargo clean
	rm -rf build

install: build
	@echo "+ Installing operating system [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py build/$(KERN).elf

test:
	cargo test --target=$(shell $(ROOT)/bin/get-host-target.sh)
