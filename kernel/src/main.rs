#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]


use core::panic::PanicInfo;
use kernel::{init, program_loader, thread_management};
use os_in_rust_common::domain::LbaAddr;
use os_in_rust_common::constants;
use os_in_rust_common::{context::BootContext, printkln};


#[inline(never)]
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {

    // 初始化一切
    init::init_all(boot_info);


    // 读取并且写入用户进程
    program_loader::sync_program(LbaAddr::new(250), 100 * constants::DISK_SECTOR_SIZE, "/cat");
    program_loader::sync_program(LbaAddr::new(350), 50 * constants::DISK_SECTOR_SIZE, "/grep");
    program_loader::sync_program(LbaAddr::new(400), 10 * constants::DISK_SECTOR_SIZE, "/echo");
    program_loader::sync_program(LbaAddr::new(410), 1443, "/main.rs");

    loop {
        thread_management::thread_yield();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printkln!("panic, {}", info);
    loop {}
}