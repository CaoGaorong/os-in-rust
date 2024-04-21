#![no_std]
#![no_main]

use core::panic::PanicInfo;

use os_in_rust_common::{paging, println};
// use core::fmt::Write;
// use os_in_rust_common::{gdt::GlobalDecriptorTable, interrupt, reg_cr0, selector, vga:: {self, CharAttr, Color, ScreenBuffer, Writer, WRITER}};

mod protect_mode;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    // println!("loader2 entered");
    paging::fill_table_directory();
    paging::fill_kernel_directory();
    paging::fill_table0();
    println!("fill table directory:");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}