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
mod mem_block;


use core::{arch::asm, mem::{self, size_of}, panic::PanicInfo};
use mem_block::Arena;
use os_in_rust_common::{constants, context::BootContext, instruction::enable_interrupt, printk, printkln};
use mutex::Mutex;
use thread::ThreadArg;


static PROCESS_NAME: &str = "user process";

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    printkln!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    // thread_management::print_thread();

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

    printkln!("size of: 0x{:x}", size_of::<Arena>());
    
    let page_size: usize = 4 * 1024;
    let addr1 = memory::malloc_kernel_page(1);
    printkln!("page addr: 0x{:x}", addr1);

    // 分配1块，新页
    let addr2 = memory::malloc(33);
    printkln!("addr: 0x{:x}, {}", addr2, addr2 == addr1 + size_of::<mem_block::Arena>() + page_size);

    // 分配1块，新页
    let addr3 = memory::malloc(12);
    printkln!("addr: 0x{:x}, {}", addr3, addr3 == addr2 + page_size);
    
    // 分配2页。新页
    let addr4 = memory::malloc(4096);
    printkln!("addr: 0x{:x}, {}", addr4, addr4 == addr3 +  page_size);
    
    // 分配1块，新页
    let addr5 = memory::malloc(129);
    printkln!("addr: 0x{:x}, {}", addr5,  addr5 == addr4 + 2 * page_size);

    // 分配1块，旧页
    let addr6 = memory::malloc(33);
    printkln!("addr: 0x{:x}, {}", addr6, addr6 == addr2 + 64);

    mem_block::get_kernel_mem_block_allocator().print_container();

    loop {
        // console_print!("k");
        // dummy_sleep(10000);
    }
}

/**
 * 用户进程
 */
extern "C" fn u_prog_a() {
    let pid = sys_call_proxy::get_pid();
    println!("user process pid: {}", pid);
    
    loop {
        // print!("u");
        // dummy_sleep(10000);
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