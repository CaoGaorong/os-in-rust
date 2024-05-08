#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;
mod init;
mod main_thread;

use core::{arch::asm, mem, panic::PanicInfo};
use os_in_rust_common::{context::BootContext, print, println, thread};


fn k_thread_fun(arg: &'static str) {
    loop {
        print!("{}", arg);
    }
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);

    let current_thread = thread::current_thread();
    println!("main thread pcb:0x{:x}", current_thread as *const _ as u32);
    thread::thread_start("my-thread", 31, k_thread_fun, "fuck");

    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}