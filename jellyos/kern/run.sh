#!/bin/bash

# Create a temporary file to store QEMU output
QEMU_OUTPUT=$(mktemp)

# Run QEMU in the background and capture the output
(make -C ../boot qemu &) >> $QEMU_OUTPUT

# Give QEMU some time to start and output the TTY path
sleep 2

# Capture the TTY output from QEMU
TTY_PATH=$(grep -oE "/dev/ttys[0-9]+" "$QEMU_OUTPUT")

# Check if TTY_PATH was found
if [ -z "$TTY_PATH" ]; then
    echo "Error: TTY path not found."
    cat "$QEMU_OUTPUT"
    rm "$QEMU_OUTPUT"
    exit 1
fi

# Define the kernel binary path (adjust as necessary)
KERN_BIN="build/kernel.bin"

# Transmit the binary to the TTY
echo "+ Transmitting $KERN_BIN to $TTY_PATH"
ttywrite -i $KERN_BIN $TTY_PATH

# Start screen with the detected TTY path
screen $TTY_PATH 115200

# Wait for the user to quit the screen session
wait

# Kill the QEMU process
pkill -f 'qemu'

# Clean up the temporary file
rm "$QEMU_OUTPUT"
