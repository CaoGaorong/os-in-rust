ENTRY(_start)

SECTIONS {
    . = 0xc048000;
    .start :
    {
        *(.start .start.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }
    .bss :
    {
         *(.bss .bss.*)
    }

}
