#!/bin/sh
rustup show active-toolchain | cut -d- -f2- | cut -d' ' -f1