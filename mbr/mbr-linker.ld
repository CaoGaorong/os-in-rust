ENTRY(_start)

SECTIONS {
    . = 0x7c00;
    .boot :
    {
        *(.boot .boot.*)
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

    . = 0x7c00 + 510;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }
}
