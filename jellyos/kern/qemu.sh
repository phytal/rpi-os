!/bin/bash
# cortex a9 has two uarts hence two serial
qemu-system-aarch64 \
    -nographic \
    -M raspi3b \
    -serial null -serial mon:stdio \
    -kernel build/kernel.elf \
    "$@"

# qemu-system-aarch64  \   -nographic  \   -M raspi3b   \  -serial null -serial mon:stdio   -kernel build/kernel.elf