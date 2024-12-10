#!/bin/bash -e

IMG=fs.img
MNT=mnt
CACHE=cache
PROGS=($(ls code/build/*.bin | xargs -n 1 basename | sed 's/.bin//'))

# Remove existing image
rm -f $IMG

# Create the image file
dd if=/dev/zero of=$IMG bs=1m count=128

# Attach the image and capture the device
DEVICE=$(hdiutil attach -imagekey diskimage-class=CRawDiskImage -nomount $IMG | awk '{print $1}' | tr -d '[:space:]')

echo "Attached device: $DEVICE"

# Ensure cleanup on exit
trap "sudo umount $MNT; hdiutil detach $DEVICE; rm -rf $MNT" EXIT

# Partition the disk and format it as FAT32
diskutil partitionDisk $DEVICE 1 MBR FAT32 "PARTITION" R

# Mount the partition
PARTITION="${DEVICE}s1"
sudo diskutil unmount $PARTITION
mkdir -p $MNT
sudo mount -t msdos $PARTITION $MNT

# Create the cache directory
mkdir -p $CACHE

# Create the programs directory on the mounted partition
mkdir -p $MNT/programs

# Build the binaries and copy them to the mounted partition
make -C code
for prog in "${PROGS[@]}"; do
    echo "Copying $prog.bin to $MNT/programs/"
    sudo cp code/build/$prog.bin $CACHE/$prog.bin
    sudo cp $CACHE/$prog.bin $MNT/programs/$prog.bin
done
