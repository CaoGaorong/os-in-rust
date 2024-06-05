use core::{slice, str};

use os_in_rust_common::{println, ASSERT};

use crate::{sys_call::{self, register_handler, HandlerType, SystemCallNo}, thread};

/**
 * 这里是系统调用暴露出去的接口
 */

/**
 * 初始化系统调用接口
 */
pub fn init() {
    // getPid
    sys_call::register_handler(SystemCallNo::GetPid, HandlerType::NoneParam(get_pid));

    // write
    sys_call::register_handler(SystemCallNo::Write, HandlerType::TwoParams(write));
}

/**
 * 获取当前任务的pid
 */
pub fn get_pid() -> u32 {
    thread::current_thread().task_struct.pid as u32
}

pub fn write(addr: u32, len: u32) -> u32 {
    
    let str_res = str::from_utf8(unsafe { slice::from_raw_parts(addr as *const u8, len as usize) });
    ASSERT!(str_res.is_ok());
    let string = str_res.unwrap();
    println!("{}", string);
    string.len() as u32
}
