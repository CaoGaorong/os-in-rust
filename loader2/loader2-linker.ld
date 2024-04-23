ENTRY(_start)

SECTIONS {
    . = 0xb00;
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
    . = 0x1300;
    
    /* 指定页目录表的地址 1MB之上。大小4KB(0x1000) */
    . = 0x100000;
    .page_dir : AT(ADDR(.page_dir)) { *(.page_dir) }

    /* 指定页表的地址，处在页目录表的上面 */
    . = 0x100000 + 0x1000;
    .page_table_list : AT(ADDR(.page_table_list)) { *(.page_table_list) }

}
