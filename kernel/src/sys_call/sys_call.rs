use core::ptr;

use os_in_rust_common::{constants, racy_cell::RacyCell};

use crate::{sys_call::sys_call_api, thread};

/**
 * 关于系统调用的实现
 */

/**
 * 系统调用函数数组
 */
pub static SYSTEM_CALL_TABLE: RacyCell<[Option<HandlerType>; constants::SYSTEM_CALL_HANDLER_CNT]> = RacyCell::new([Option::None; constants::SYSTEM_CALL_HANDLER_CNT]);

/**
 * 注册系统调用函数
 */
#[inline(never)]
pub fn register_handler(sys_call_no: SystemCallNo, handler: HandlerType) {
    let system_call_table = unsafe { SYSTEM_CALL_TABLE.get_mut() };
    system_call_table[sys_call_no as usize] = Option::Some(handler);
}

/**
 * 根据系统调用号，得到系统调用的函数
 */
pub fn get_handler(sys_call_no: u32) -> Option<HandlerType> {
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
    /**
     * I/O写入数据
     */
    Write = 0x01,

    /**
     * 打印字符
     */
    Print = 0x02,

    /**
     * 申请内存空间
     */
    Malloc = 0x03,

    /**
     * 释放内存空间
     */
    Free = 0x04,
    
    /**
     * fork进程
     */
    Fork = 0x05,

    /**
     * 挂起当前任务
     */
    Yield = 0x06,

    /**
     * 清除屏幕
     */
    ClearScreen = 0x07,
    
    /**
     * 读取目录
     */
    ReadDir = 0x08,
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
