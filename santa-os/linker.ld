/*
 * Based on bzt's raspi3-tutorial linker script.
 *
 * The firmware (and QEMU) load kernel8.img at 0x80000, so the image must be
 * linked to start there. The previous script put .text.boot at 0x0 and the
 * rest at 0x80000, which made objcopy emit 512 KiB of padding and shifted the
 * whole kernel by +0x80000 at runtime (vectors ended up at 0x100000).
 */

SECTIONS
{
    . = 0x80000;
    __text_beg = .;

    .text : {
        KEEP(*(.text.boot))
        KEEP(*(.text.vectors))
        *(.text .text.* .gnu.linkonce.t*)
    }

    .rodata : {
        *(.rodata .rodata.* .gnu.linkonce.r*)
    }

    PROVIDE(_data = .);
    .data : {
        *(.data .data.* .gnu.linkonce.d*)
    }

    .bss (NOLOAD) : {
        . = ALIGN(32);
        __bss_beg = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    }

    __text_end = ALIGN(8);
    __bss_len = (__bss_end - __bss_beg);
    __text_len = (__text_end - __text_beg);

    _end = .;

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
