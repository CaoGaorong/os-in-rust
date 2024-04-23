#![no_std]
#![no_main]

use core::panic::PanicInfo;

use os_in_rust_common::println;



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    println!("i'm Kernel!");
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}