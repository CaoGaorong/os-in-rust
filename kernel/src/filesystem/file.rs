use os_in_rust_common::{constants, racy_cell::RacyCell};

use super::constant;

/**
 * 整个系统打开的文件
 */
const REPEAT_FILE:Option<OpenedFile> = Option::None;
static GLOBAL_FILE_TABLE: RacyCell<[Option<OpenedFile>; constant::MAX_OPENED_FILE_IN_SYSTEM]> = RacyCell::new([REPEAT_FILE; constant::MAX_OPENED_FILE_IN_SYSTEM]);


/**
 * 文件系统的，文件的结构
 */
pub struct OpenedFile {
    
}

/**
 * 标准文件描述符
 */
pub enum StdFileDescriptor {
    /**
     * 标准输入
     */
    StdInputNo = 0x0,
    /**
     * 标准输出
     */
    StdOutputNo = 0x1,
    /**
     * 标准错误
     */
    StdErrorNo = 0x2,
}