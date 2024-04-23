#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

use os_in_rust_common::dap;

global_asm!(include_str!("boot.s"));


#[no_mangle]
pub extern "C" fn main() {

    // 把loader从磁盘加载到内存
    dap::load_disk(2, 1, 0x900);

    let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(0x900 as *const ()) };
    loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
