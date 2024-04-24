#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
mod protect_mode;
use os_in_rust_common::{dap, disk, selector::SegmentSelector, constants};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // 进入保护模式
    protect_mode::enter_protect_mode();
    
    // 把loader2从磁盘加载到内存
    disk::read_disk(constants::LOADER2_LBA, constants::LOADER2_SEC_CNT as u8, constants::LOADER2_ADDR);

    // let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(0xb00 as *const ()) };
    // let selector = SegmentSelector::Code0Selector as u16;
    unsafe {
        asm!(
            "jmp 0x8, 0xb00"
        );
    }

    // loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}