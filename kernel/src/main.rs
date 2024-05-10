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
        // println!("sub thread, intr:{}", instruction::is_intr_on());
    }
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 启动一个子线程。不执行
    thread_management::thread_start("my-thread", 31, k_thread_fun, ".");

    // 打印线程信息
    thread_management::print_thread();
    
    enable_interrupt();
    loop {
        // println!("main thread interrupt on {}", instruction::is_intr_on());
        // print!("-");
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}