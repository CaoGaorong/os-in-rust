#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::println;


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    println!("Hello, I'm user process");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}