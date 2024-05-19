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
mod keyboard;
mod scancode;
mod printer;
pub mod blocking_queue;
pub mod tss;



use core::{arch::asm, mem, panic::PanicInfo, ptr};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, print, println, queue::Queue, racy_cell::RacyCell, thread::{self, current_thread}};
use sync::Lock;
use mutex::Mutex;

use crate::blocking_queue::{ArrayBlockingQueue, BlockingQueue};

/**
 * 子线程，无限循环，取出键盘元素
 */
fn keycode_consumer(arg: &'static str) {
    loop {
        // 从阻塞队列中取出键码
        let key_opt = keyboard::get_keycode_queue().take();
        // 把键码，调用打印程序打印出来
        printer::print_key_code(key_opt);
    }
}


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    thread_management::print_thread();

    // 这里创建子线程，特意把priority设置为1，而main线程的priority设置的是5
    thread_management::thread_start("thread_a", 1, keycode_consumer, "!");

    enable_interrupt();

    loop {}
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