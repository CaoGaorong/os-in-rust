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
pub mod memory;
pub mod page_util;
pub mod process;
mod thread;


use core::{arch::asm, mem, panic::PanicInfo, ptr};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, print, println, queue::Queue, racy_cell::RacyCell};
use sync::Lock;
use mutex::Mutex;

use crate::blocking_queue::{ArrayBlockingQueue, BlockingQueue};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    thread_management::print_thread();

    // 这里创建子线程，特意把priority设置为1，而main线程的priority设置的是5
    // thread_management::thread_start("thread_a", 1, keycode_consumer, "!");

    process::process_execute("user process", u_prog_a);

    enable_interrupt();

    loop {}
}


fn u_prog_a() {
    loop {
       println!("user process");
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