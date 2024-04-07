#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_in_rust_common::vga:: {Writer, CharAttr, Color, ScreenBuffer};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    let mut writer = Writer::new(unsafe { &mut *(0xb8000 as *mut ScreenBuffer) }, CharAttr::new(Color::White, Color::Black, false));
    // 正常输出
    writer.write_byte(b'H');
    writer.write_byte(b'e');
    writer.write_byte(b'l');
    writer.write_byte(b'l');
    writer.write_byte(b'o');

    // 如果去掉lto=true，则无法输出
    for ele in "Hello".bytes() {
        writer.write_byte(ele);
    }

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}