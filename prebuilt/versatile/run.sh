qemu-system-aarch64 \
  -M raspi3b \
  -kernel kernel.img \
  -dtb versatile-pb.dtb \
  -m 1024 \
  -serial stdio \
  -d int,cpu_reset
