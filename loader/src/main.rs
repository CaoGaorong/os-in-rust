#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_in_rust_common::vga:: {Writer, CharAttr, Color, ScreenBuffer};
mod protect_mode;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // 进入保护模式
    protect_mode::enter_protect_mode();

    // 打印一下
    let mut writer = Writer::new(unsafe { &mut *(0xb8000 as *mut ScreenBuffer) }, CharAttr::new(Color::White, Color::Black, false));
    writer.write_string("Fuck you");

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}