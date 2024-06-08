#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]

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
mod sys_call;
mod pid_allocator;
mod sys_call_api;
mod sys_call_proxy;


use core::{arch::asm, mem, panic::PanicInfo, ptr, sync::atomic::{AtomicU8, Ordering}};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, printk, printkln, queue::Queue, racy_cell::RacyCell, vga::print};
use sync::Lock;
use mutex::Mutex;
use thread::ThreadArg;

use crate::blocking_queue::{ArrayBlockingQueue, BlockingQueue};

static PROCESS_NAME: &str = "user process";

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    printkln!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    thread_management::print_thread();

    process::process_execute(PROCESS_NAME, u_prog_a);
    thread_management::thread_start("thread_a", 5, kernel_thread, 0);

    printkln!("-----system started-----");
    printkln!();

    enable_interrupt();
    loop {}
}


/**
 * 内核线程
 */
extern "C" fn kernel_thread(arg: ThreadArg) {
    let pid = sys_call_proxy::get_pid();
    printkln!("kernel thread pid:{}", pid);


    loop {}
}

/**
 * 用户进程
 */
extern "C" fn u_prog_a() {
    let pid = sys_call_proxy::get_pid();
    println!("user process pid: {}", pid);
    
    loop {
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