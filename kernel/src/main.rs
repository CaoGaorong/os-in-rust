#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupt;
mod init;
mod thread_management;
mod scheduler;

use core::{arch::asm, mem, panic::PanicInfo};
use os_in_rust_common::{constants, context::BootContext, instruction::{self, enable_interrupt}, print, println, thread::{self, current_thread}};


fn k_thread_fun(arg: &'static str) {
    loop {
        print!("{}", arg);
        // 防止打印得太快了，sleep一下
        dummy_sleep(100000);
    }
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 创建线程，假如就绪队列
    thread_management::thread_start("thread_a", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "!");
    thread_management::thread_start("thread_b", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "@");
    thread_management::thread_start("thread_c", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "#");
    thread_management::thread_start("thread_d", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "$");
    thread_management::thread_start("thread_e", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "%");
    thread_management::thread_start("thread_f", constants::TASK_DEFAULT_PRIORITY, k_thread_fun, "(");

    // 打印线程信息
    thread_management::print_thread();
    
    enable_interrupt();
    loop {
        // println!("main thread interrupt on {}", instruction::is_intr_on());
        print!("-");
        dummy_sleep(100000);
    }
}

/**
 * 做一个假的sleep
 */
fn dummy_sleep(instruction_cnt: u32) {
    for _ in 0 .. instruction_cnt {
        unsafe {asm!("nop");}
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}