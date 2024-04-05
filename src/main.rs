#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo, ptr};
global_asm!(include_str!("boot.s"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    let hello = b"Hello, World";
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &e) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = e;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
