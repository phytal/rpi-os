qemu-system-arm \
  -M versatilepb \
  -cpu arm1176 \
  -m 256 \
  -kernel kernel.img \
  -dtb versatile-pb-buster-5.4.51.dtb \
  -serial stdio
  -nographic

