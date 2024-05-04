#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{context::{self, BootContext}, mem, println, ASSERT, MY_PANIC};



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: u32) {
    println!("I'm Kernel!");
    
    // 初始化中断描述符和中断控制器
    interrupt::init();
    
    // 开启中断
    // instruction::enable_interrupt();
    // println!("0x{:x}", boot_info);

    // 抛出断言
    let boot_context = unsafe { *(boot_info as *const BootContext) };
    let len = boot_context.memory_map_len;
    println!("0x{:x}", len);
    
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}