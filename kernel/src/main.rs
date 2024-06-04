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


use core::{arch::asm, mem, panic::PanicInfo, ptr, sync::atomic::{AtomicU8, Ordering}};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, print, println, queue::Queue, racy_cell::RacyCell};
use sync::Lock;
use mutex::Mutex;
use thread::ThreadArg;

use crate::blocking_queue::{ArrayBlockingQueue, BlockingQueue};

static PROCESS_NAME: &str = "user process";

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    println!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    thread_management::print_thread();

    thread_management::thread_start("thread_a", 5, kernel_thread, 0);
    process::process_execute(PROCESS_NAME, u_prog_a);

    enable_interrupt();
    loop {}
}

static NUM: RacyCell<AtomicU8> = RacyCell::new(AtomicU8::new(0));

extern "C" fn u_prog_a() {
    loop {
        unsafe { NUM.get_mut() }.fetch_add(1, Ordering::Release);
        dummy_sleep(10000000);
        
        // 用户进程不能调用内核程序，不能直接输出
        // println!("user process");
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


extern "C" fn kernel_thread(arg: ThreadArg) {
    print_pid();
    // let task = &thread::current_thread().task_struct;
    // println!("task name: {}, task pid: {}", task.name as &str, task.pid as u8);

    loop {
        println!("{}", unsafe { NUM.get_mut() }.load(Ordering::Acquire));
        dummy_sleep(10000000);
    }
}

#[inline(never)]
fn print_pid() {
    let mut pid: u32;
    unsafe {
        asm!(
            "mov eax, 0x0",
            "int 0x80",
            "mov eax, eax",
            out("eax") pid,
        )
    }
    println!("pid:{}", pid);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}