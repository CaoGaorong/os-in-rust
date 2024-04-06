#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    let hello = b"I'm Loader";
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &e) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = e;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}