#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;
mod init;

use core::{arch::asm, mem, panic::PanicInfo};
use os_in_rust_common::{context::BootContext, println};



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);

    

    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}