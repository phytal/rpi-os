# Outer Makefile
INTERNAL_DIR := code
IMG := fs.img
MNT := mnt
CACHE := cache

.PHONY: all build fs clean

# Default target - build binaries and filesystem
all: build fs

# Call the internal Makefile to build the binaries
build:
	@echo "+ Running internal Makefile in $(INTERNAL_DIR)"
	$(MAKE) -C $(INTERNAL_DIR) all

# Create the filesystem image and copy the binaries
fs: build
	./build.sh

# Clean target - clean internal build and filesystem
clean:
	@echo "+ Cleaning both internal and filesystem artifacts"
	$(MAKE) -C $(INTERNAL_DIR) clean
	rm -rf $(IMG) $(MNT) $(CACHE)

