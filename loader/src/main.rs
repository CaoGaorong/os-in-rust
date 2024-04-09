#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_in_rust_common::vga:: {Writer, CharAttr, Color, ScreenBuffer};
use os_in_rust_common::gdt::SegmentDescritor;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    let mut writer = Writer::new(unsafe { &mut *(0xb8000 as *mut ScreenBuffer) }, CharAttr::new(Color::White, Color::Black, false));

    for i in 0 .. 2000 {
        writer.write_byte(b'a');
    }
    for i in  0 .. 10 {
        writer.write_string("Hello, world");
    }
    
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}