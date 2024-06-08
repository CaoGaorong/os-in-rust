use core::arch::asm;
use core::fmt;
use os_in_rust_common::vga::WRITER;

use crate::sys_call::SystemCallNo;

/**
 * 给用户程序，发起系统调用
 */
pub fn get_pid() -> u8 {
    do_sys_call(SystemCallNo::GetPid as u32, Option::None, Option::None, Option::None) as u8
}

/**
 * 系统调用 print!
 */
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys_call_proxy::sys_print(format_args!($($arg)*)));
}

/**
 * 系统调用 println!
 */
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/**
 * 给定Arguments，然后发起系统调用
 */
#[no_mangle]
pub fn sys_print(args: fmt::Arguments) {
    // 取出参数的地址
    let arg_addr = &args as *const _ as u32;
    // 调用系统调用，把参数地址传过去
    do_sys_call(SystemCallNo::Print as u32, Option::Some(arg_addr), Option::None, Option::None);
}

/**
 * 写入字符
 */
pub fn write(string: &str) {
    do_sys_call(SystemCallNo::Write as u32, Option::Some(string.as_ptr() as usize as u32), Option::Some(string.len() as u32), Option::None);
}


/**
 * 发起系统调用
 * eax: 系统调用号
 * ebx_opt：第一个参数；可选
 * ecx_opt：第二个参数；可选
 * edx_opt：第三个参数；可选
 */
#[inline(always)]
fn do_sys_call(eax: u32, ebx_opt: Option<u32>, ecx_opt: Option<u32>, edx_opt: Option<u32>) -> u32 {
    let res: u32;
    unsafe {
        asm!("mov eax, eax", in("eax") eax);
        if ebx_opt.is_some() {
            let ebx = ebx_opt.unwrap();
            asm!("mov ebx, ebx", in("ebx")ebx );
        }
        if ecx_opt.is_some() {
            let ecx = ecx_opt.unwrap();
            asm!("mov ecx, ecx", in("ecx")ecx );
        }
        if edx_opt.is_some() {
            let edx = edx_opt.unwrap();
            asm!("mov edx, edx", in("edx")edx );
        }
        asm!(
            "int 0x80",
            "mov eax, eax",
            out("eax") res,
        );
    }
    res
}