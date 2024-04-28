#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{instruction, println};



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    println!("I'm Kernel!");
    // 初始化
    interrupt::init();
    
    instruction::enable_interrupt();
    // println!("waiting....");
    
    unsafe {asm!("hlt");}
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}