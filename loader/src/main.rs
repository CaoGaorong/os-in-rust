#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
mod protect_mode;
use os_in_rust_common::{dap, selector::SegmentSelector};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // 把loader2从磁盘加载到内存
    dap::load_disk(3, 4, 0xb00);

    // dap::load_disk(7, 400, 0x1500);

    let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(0xb00 as *const ()) };
    protect_mode::enter_protect_mode();
    let selector = SegmentSelector::Code0Selector as u16;
    unsafe {
        asm!(
            "jmp 0x8, 0xb00"
        );
    }

    loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}