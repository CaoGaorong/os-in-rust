use core::ptr;

use os_in_rust_common::{constants, racy_cell::RacyCell};

use crate::thread;

/**
 * 关于系统调用的实现
 */

/**
 * 系统调用函数数组
 */
pub static SYSTEM_CALL_TABLE: RacyCell<[HandlerType; constants::SYSTEM_CALL_HANDLER_CNT]> = RacyCell::new([HandlerType::NoneParam(||0); constants::SYSTEM_CALL_HANDLER_CNT]);

/**
 * 初始化系统调用函数
 */
pub fn init() {
    let system_call_table = unsafe { SYSTEM_CALL_TABLE.get_mut() };
    system_call_table[SystemCallNo::GetPid as usize] = HandlerType::NoneParam(get_pid);
}

/**
 * 根据系统调用号，得到系统调用的函数
 */
pub fn get_handler(sys_call_no: u32) -> HandlerType {
    let system_call_table = unsafe { SYSTEM_CALL_TABLE.get_mut() };
    system_call_table[sys_call_no as usize]
}

/**
 * 系统调用号枚举
 */
#[derive(Clone, Copy)]
pub enum SystemCallNo {
    /**
     * 获取当前任务的PID
     */
    GetPid = 0x0,
}

/**
 * 系统调用的类型（根据参数个数来区分）
 */
#[derive(Clone, Copy)]
pub enum HandlerType {
    /**
     * 不需要参数的系统调用函数
     */
    NoneParam(fn() -> u32),
    /**
     * 需要1个参数的系统调用函数
     */
    OneParam(fn(u32) -> u32),
    /**
     * 需要2个参数的系统调用函数
     */
    TwoParams(fn(u32, u32) -> u32),
    /**
     * 需要3个参数的系统调用函数
     */
    ThreeParams(fn(u32, u32, u32) -> u32),
}


/**
 * 获取当前任务的pid
 */
fn get_pid() -> u32 {
    thread::current_thread().task_struct.pid as u32
}