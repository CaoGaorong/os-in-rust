#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;
mod init;
mod thread_management;
mod scheduler;

use core::{arch::asm, mem, panic::PanicInfo};
use os_in_rust_common::{context::BootContext, instruction::{self, enable_interrupt}, print, println, thread::{self, current_thread}};


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
    thread_management::init_list();
    
    // 启动一个子线程。不执行
    thread_management::thread_start("my-thread", 31, k_thread_fun, "fuck");
    // 主线程
    thread_management::make_thread_main();

    // 打印线程信息
    thread_management::print_thread();
    
    enable_interrupt();
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}