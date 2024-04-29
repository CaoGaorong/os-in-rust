#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{println, ASSERT};



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    println!("I'm Kernel!");
    
    // 初始化中断描述符和中断控制器
    interrupt::init();
    
    // 开启中断
    // instruction::enable_interrupt();
    
    // 抛出断言
    ASSERT!(1 == 2);

    
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}