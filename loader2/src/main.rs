#![no_std]
#![no_main]

use core::panic::PanicInfo;

use os_in_rust_common::{gdt::{self, GlobalDecriptorTable}, paging, println};
// use core::fmt::Write;
// use os_in_rust_common::{gdt::GlobalDecriptorTable, interrupt, reg_cr0, selector, vga:: {self, CharAttr, Color, ScreenBuffer, Writer, WRITER}};


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    // println!("loader2 entered");
    paging::fill_table_directory();
    paging::fill_kernel_directory();
    paging::fill_table0();

    // 取出GDT的地址
    let gdt_addr = gdt::get_gdt_addr();
    
    gdt::load_gdtr_by_addr((gdt_addr as u32 + 0xc0000000) as *const GlobalDecriptorTable);

    println!("fill table directory:");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}