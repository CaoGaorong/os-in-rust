use core::arch::asm;
use core::fmt;
use os_in_rust_common::vga::WRITER;

use crate::pid_allocator::Pid;

use super::sys_call::SystemCallNo;

/**
 * 本模块理论上是系统库提供给用户进程使用了，不是操作系统的实现提供的。
 * 比如gcc库给C用户程序提供的发起系统调用的包
 */


/**
 * 系统调用 print!
 */
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys_call::sys_call_proxy::sys_print(format_args!($($arg)*)));
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
 * 获取当前任务的pid
 */
pub fn get_pid() -> u8 {
    do_sys_call(SystemCallNo::GetPid, Option::None, Option::None, Option::None) as u8
}


/**
 * 给定Arguments，然后发起系统调用
 */
#[no_mangle]
pub fn sys_print(args: fmt::Arguments) {
    // 取出参数的地址
    let arg_addr = &args as *const _ as u32;
    // 调用系统调用，把参数地址传过去
    do_sys_call(SystemCallNo::Print, Option::Some(arg_addr), Option::None, Option::None);
}

/**
 * 写入字符
 */
pub fn write(string: &str) {
    do_sys_call(SystemCallNo::Write, Option::Some(string.as_ptr() as usize as u32), Option::Some(string.len() as u32), Option::None);
}

/**
 * 发起系统调用，申请bytes大小的内存空间
 */
pub fn malloc<T>(bytes: usize) -> *mut T {
    let addr = do_sys_call(SystemCallNo::Malloc, Option::Some(bytes as u32), Option::None, Option::None);
    addr as *mut T
}

/**
 * 发起系统调用，释放内存空间
 */
pub fn free<T>(ptr: *const T) {
    let addr = ptr as u32;
    do_sys_call(SystemCallNo::Free, Option::Some(addr as u32), Option::None, Option::None);
}

pub enum ForkResult {
    Parent(Pid),
    Child
}

pub fn fork() -> ForkResult {
    // 调用系统调用的fork
    let fork_res = self::do_sys_call(SystemCallNo::Fork, Option::None, Option::None, Option::None);
    if fork_res == 0 {
        ForkResult::Child
    } else {
        ForkResult::Parent(Pid::new(fork_res.try_into().unwrap()))
    }
}

/**
 * 发起系统调用
 * eax: 系统调用号
 * ebx_opt：第一个参数；可选
 * ecx_opt：第二个参数；可选
 * edx_opt：第三个参数；可选
 */
#[inline(always)]
#[cfg(all(not(test), target_arch = "x86"))]
fn do_sys_call(eax_sys_nr: SystemCallNo, ebx_opt: Option<u32>, ecx_opt: Option<u32>, edx_opt: Option<u32>) -> u32 {
    let eax = eax_sys_nr as u32;
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
#[cfg(all(not(target_arch = "x86")))]
fn do_sys_call(eax_sys_nr: SystemCallNo, ebx_opt: Option<u32>, ecx_opt: Option<u32>, edx_opt: Option<u32>) -> u32 {
    todo!()
}