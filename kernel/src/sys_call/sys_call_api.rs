use core::{fmt, slice, str};

use os_in_rust_common::{printkln, ASSERT};

use crate::{console, fork, memory, thread};
use super::sys_call::{self, HandlerType, SystemCallNo};

/**
 * 这里是系统调用暴露出去的接口
 */

/**
 * 初始化系统调用接口
 */
#[inline(never)]
pub fn init() {
    // getPid
    sys_call::register_handler(SystemCallNo::GetPid, HandlerType::NoneParam(get_pid));

    // write
    sys_call::register_handler(SystemCallNo::Write, HandlerType::TwoParams(write));

    // println
    sys_call::register_handler(SystemCallNo::Print, HandlerType::OneParam(print_format));
    
    // malloc
    sys_call::register_handler(SystemCallNo::Malloc, HandlerType::OneParam(malloc));
    
    // free
    sys_call::register_handler(SystemCallNo::Free, HandlerType::OneParam(free));
    
    // fork
    sys_call::register_handler(SystemCallNo::Fork, HandlerType::NoneParam(fork));
}

/**
 * 获取当前任务的pid
 */
fn get_pid() -> u32 {
    let cur_task = &thread::current_thread().task_struct;
    cur_task.pid.get_data().try_into().unwrap()
}

/**
 * write系统调用
 */
fn write(addr: u32, len: u32) -> u32 {
    
    let str_res = str::from_utf8(unsafe { slice::from_raw_parts(addr as *const u8, len as usize) });
    ASSERT!(str_res.is_ok());
    let string = str_res.unwrap();
    printkln!("{}", string);
    string.len() as u32
}

/**
 * 打印系统调用
 */
fn print_format(argument_addr: u32) -> u32 {
    // 把参数地址，转成对象
    let arg = unsafe { *(argument_addr as *const fmt::Arguments) };
    
    // 使用console_print函数，打印
    console::console_print(arg);
    0
}

/**
 * 申请bytes大小的内存空间
 */
fn malloc(bytes: u32) -> u32 {
    memory::sys_malloc(bytes as usize) as u32
}

/**
 * 释放某个地址的内存空间
 */
fn free(addr_to_free: u32) -> u32 {
    memory::sys_free(addr_to_free.try_into().unwrap());
    0
}


/**
 * fork
 */
fn fork() -> u32 {
    fork::fork().get_data().try_into().unwrap()
}