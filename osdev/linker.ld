ENTRY(_start) 
SECTIONS
{
    /* Starts at LOADER_ADDR. */
    . = 0x8000; /* sets current address */
    /* For AArch64, use . = 0x80000; */
    __start = .;
    /* section for code */
    __text_start = .; 
    .text :
    {
        KEEP(*(.text.boot))
        *(.text)
    }
    . = ALIGN(4096); /* align to page size */
    __text_end = .;

    /* read only data section */
    __rodata_start = .; 
    .rodata :
    {
        *(.rodata)
    }
    . = ALIGN(4096); /* align to page size */
    __rodata_end = .;

    /* rw data section */
    __data_start = .;
    .data :
    {
        *(.data)
    }
    . = ALIGN(4096); /* align to page size */
    __data_end = .;

    /* bss section (0 initialized memory) */
    __bss_start = .;
    .bss :
    {
        bss = .;
        *(.bss)
    }
    . = ALIGN(4096); /* align to page size */
    __bss_end = .;
    __bss_size = __bss_end - __bss_start;
    __end = .;
}