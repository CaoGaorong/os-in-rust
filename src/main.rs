#![no_std]
#![no_main]
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
global_asm!(include_str!("boot.s"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    unsafe {
        *vga_buffer.offset(0) = b'H';
        *vga_buffer.offset(1) = 0xb;
        *vga_buffer.offset(2) = b'e';
        *vga_buffer.offset(3) = 0xb;
        *vga_buffer.offset(4) = b'l';
        *vga_buffer.offset(5) = 0xb;
        *vga_buffer.offset(6) = b'l';
        *vga_buffer.offset(7) = 0xb;
        *vga_buffer.offset(8) = b'0';
        *vga_buffer.offset(9) = 0xb;
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
