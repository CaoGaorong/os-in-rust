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


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    thread_management::print_thread();
    
    enable_interrupt();
    loop {
        // my_print("-");
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}