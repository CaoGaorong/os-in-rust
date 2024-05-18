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

use core::{arch::asm, mem, panic::PanicInfo, ptr};
use lazy_static::lazy_static;
use os_in_rust_common::{constants, context::BootContext, instruction::{self, disable_interrupt, enable_interrupt}, print, println, queue::Queue, racy_cell::RacyCell, thread::{self, current_thread}};
use sync::Lock;
use mutex::Mutex;

use crate::blocking_queue::{ArrayBlockingQueue, BlockingQueue};

static mut BUFFER: [u8; 4] = [0;4];
static BLOCKING_QUEUE: RacyCell<ArrayBlockingQueue<u8>> = RacyCell::new(ArrayBlockingQueue::new(unsafe { &mut BUFFER }));

/**
 * 子线程，无限循环消费数据
 */
fn consumer_func(arg: &'static str) {
    // 获取阻塞队列
    let blocking_queue = unsafe { BLOCKING_QUEUE.get_mut() };
    
    // 子线程无线循环，往里面取元素。知道阻塞为止
    loop {

        // 休息一下
        dummy_sleep(100000);
        let e = blocking_queue.take();
        console_println!("thread take {} ", e);
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
    thread_management::thread_start("thread_a", 1, consumer_func, "!");

    enable_interrupt();

    let blocking_queue = unsafe { BLOCKING_QUEUE.get_mut() };
    
    // 主线程往里面放元素
    for i in 0 .. 10 {
        // 休息一下
        dummy_sleep(100000);
        blocking_queue.put(i);
        console_println!("main put {} ", i);
    }

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