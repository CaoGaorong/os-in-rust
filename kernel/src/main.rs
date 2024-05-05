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
    
    // 开启中断
    // instruction::enable_interrupt();

    println!("boot_info: 0x{:x}", boot_info as *const _ as u32);
    println!("map addr: 0x{:x}", boot_info.memory_map_addr);
    let memeory_map:&mut [AddressRangeDescriptorStrure]  = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.memory_map_addr as *mut _,
            boot_info.memory_map_len.try_into().unwrap(),
        )
    };

    memeory_map
    .iter()
    // .filter(|&m| m.region_type == ARDSType::Usable as u32)
    .for_each(|m| {
        let a = m.base_addr;
        println!("addr: 0x{:x}", a);
        let l = m.len;
        println!("len: 0x{:x}", l);
        let t = m.region_type;
        println!("type: {}", t);

        println!();
    });

    
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}