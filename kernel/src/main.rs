#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

mod interrupt;
mod init;
mod thread_management;
mod scheduler;
mod sync;
mod mutex;
mod console;

use core::{arch::asm, mem, panic::PanicInfo, ptr};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, print, println, racy_cell::RacyCell, thread::{self, current_thread}};
use sync::Lock;
use mutex::Mutex;


fn k_thread_fun(arg: &'static str) {
    loop {
        my_print(arg);
    }
}

fn my_print(arg: &'static str) {
    // 打印。可以对比一下下面两种打印，一个加了阻塞锁，一个没有加阻塞锁
    // print!("{}", arg);
    console_print!("{}", arg);
    // 防止打印得太快了，sleep一下
    dummy_sleep(100000);
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
        my_print("-");
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