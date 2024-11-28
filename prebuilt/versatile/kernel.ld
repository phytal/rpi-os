ENTRY(_start)

SECTIONS {
    . = 0x08000000; /* Starting address of your kernel in memory */
    .text : {
        *(.text)  /* Include all .text sections from object files */
    }
}

