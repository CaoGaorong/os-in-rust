#![no_std]
#![no_main]

use core::panic::PanicInfo;

use os_in_rust_common::vga::{self, CharAttr, Color, ScreenBuffer, Writer};
// use core::fmt::Write;
// use os_in_rust_common::{gdt::GlobalDecriptorTable, interrupt, reg_cr0, selector, vga:: {self, CharAttr, Color, ScreenBuffer, Writer, WRITER}};
// mod protect_mode;
// mod paging;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    let mut writer = Writer::new(
        unsafe { &mut *(0xb8000 as *mut ScreenBuffer) },
        CharAttr::new(Color::White, Color::Black, false));
    writer.write_string("fuck");
    // unsafe{WRITER.lock().write_string("FUck");}
    // WRITER.lock().write_string("FUck");
    // println!("Hello, World");
    // write!(writer, "The numbers are {} and {}", 42, 1.0/3.0);
    // 进入保护模式
    // protect_mode::enter_protect_mode();

    // writer.write_string("start\n");
    // // 填充页目录表
    // let r = paging::fill_table_directory();
    // writer.write_string("end\n");



    // writer.write_byte(r as u8);
    // writer.write_string("Fuck YOU ");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}