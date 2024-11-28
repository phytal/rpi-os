ROOT := $(shell git rev-parse --show-toplevel)
BINARIES := $(patsubst src/bin/%.rs,%,$(wildcard src/bin/*.rs)) # Automatically get all binaries in src/bin
TARGET_DIR := target/aarch64-unknown-none/release
OBJCPY := cargo objcopy --bin
OBJDUMP := cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex
NM := cargo nm

.PHONY: all build objdump nm clean

# The default target to build all binaries
all: $(BINARIES:%=build/%)

# Build each binary
build/%:
	@echo "+ Building $@.elf"
	@cargo build --release --bin $*
	@mkdir -p build
	@cp -f $(TARGET_DIR)/$* build/$*.elf
	@echo "+ Building build/$*.bin [objcopy]"
	@$(OBJCPY) $* --release -- -O binary --strip-all build/$*.bin

# Check the project (runs cargo check for all binaries)
check:
	@cargo check

# Objdump for all binaries
objdump: $(BINARIES:%=objdump/%)

objdump/%: build/%
	@$(OBJDUMP) build/$*.elf

# nm for all binaries
nm: $(BINARIES:%=nm/%)

nm/%: build/%
	@$(NM) build/$*.elf

# Clean target
clean:
	@cargo clean
	@rm -rf build
