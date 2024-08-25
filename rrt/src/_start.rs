#![no_std]
#![no_main]


use core::arch::asm;

use crate::env;

#[no_mangle]
#[link_section = ".start"]
pub fn _start() {
    // 参数的地址
    let arg_addr: u32;
    // 参数的长度
    let arg_len: u32;
    unsafe {
        asm!(
            "mov {0:e}, ebx",
            "mov {1:e}, ecx",
            out(reg) arg_addr,
            out(reg) arg_len,
        )
    }

    // 执行这个用户进程，传递的参数
    let args = unsafe { core::str::from_utf8(core::slice::from_raw_parts(arg_addr as *const u8, arg_len as usize)) };
    if args.is_ok() {
        // 如果有参数，那么就放到容器中
        env::set_args(args.unwrap());
    }

    // 调用main函数
    unsafe {
        asm!(
            "call main"
        )
    }
}