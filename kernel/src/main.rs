#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{bios_mem::AddressRangeDescriptorStrure, context::{self, BootContext}, println, ASSERT, MY_PANIC};



#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    // 初始化中断描述符和中断控制器
    interrupt::init();
    
    println!("boot_info: 0x{:x}", boot_info as *const _ as u32);
    println!("map addr: 0x{:x}", boot_info.memory_map_addr);
    println!("map len: 0x{:x}", boot_info.memory_map_len);
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}