use os_in_rust_common::{constants, racy_cell::RacyCell};
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
    Write,

    /**
     * 从I/O中读取数据
     */
    Read,

    /**
     * 打印字符
     */
    Print,

    /**
     * 申请内存空间
     */
    Malloc,

    /**
     * 释放内存空间
     */
    Free,
    
    /**
     * fork进程
     */
    Fork,

    /**
     * 挂起当前任务
     */
    Yield,

    /**
     * 清除屏幕
     */
    ClearScreen,
    
    /**
     * 读取目录
     */
    ReadDir,
    /**
     * 删除目录
     */
    RemoveDir,

    /**
     * 创建目录
     */
    CreateDir,
    /**
     * 递归创建目录
     */
    CreateDirAll,

    /**
     * 生成目录迭代器
     */
    DirIterator,

    /**
     * 遍历目录的时候用
     */
    DirIteratorNext,

    /**
     * 迭代器drop traits
     */
    DirIteratorDrop,

    /**
     * 打开一个文件
     */
    OpenFile,

    /**
     * seek文件
     */
    Seek,

    /**
     * 关闭文件
     */
    CloseFile,

    /**
     * 读取文件大小
     */
    FileSize,

    /**
     * 创建一个文件
     */
    CreateFile,

    /**
     * 写入文件
     */
    WriteFile,

    /**
     * 删除文件
     */
    RemoveFile,

    /**
     * 执行
     */
    Exec,

    /**
     * 系统调用退出
     */
    Exit,
    /**
     * 系统调用等待
     */
    Wait,

    /**
     * 获取当前任务的工作目录
     */
    Cwd,

    /**
     * 切换当前任务的工作目录
     */
    Cd,

    /**
     * 创建pipe
     */
    PipeCreate,
    /**
     * pipe结束
     */
    PipeEnd,

    /**
     * 文件描述符重定向
     */
    FileDescriptorRedirect,
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
