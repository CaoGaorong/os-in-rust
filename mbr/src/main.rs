#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

use os_in_rust_common::{dap, constants};

global_asm!(include_str!("boot.s"));


#[no_mangle]
pub extern "C" fn main() {

    // 把loader从磁盘加载到内存
    dap::load_disk(constants::LOADER_LBA as u64, constants::LOADER_SEC_CNT as u16, constants::LOADER_ADDR);

    // 找到loader的入口
    let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(constants::LOADER_ADDR as *const ()) };
    // 跳转过去执行
    loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
