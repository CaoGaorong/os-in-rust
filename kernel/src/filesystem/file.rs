use core::mem::size_of;

use os_in_rust_common::{constants, domain::InodeNo, racy_cell::RacyCell, ASSERT};

use crate::{filesystem::{dir::{self, DirEntry, FileType}, global_file_table}, init, memory, thread};

use super::{dir::Dir, fs::FileSystem, inode::{Inode, OpenedInode}};

/**
 * 文件系统的，文件的结构
 */
pub struct OpenedFile {
    /**
     * 这个打开的文件，底层指向的inode节点
     */
    inode: &'static OpenedInode,
    /**
     * 操作的文件的偏移量（单位字节）
     */
    file_off: usize,
    /**
     * 这个打开的文件的操作标识
     */
    flag: FileFlag,
}

impl OpenedFile {
    pub fn new(inode: &'static OpenedInode) -> Self {
        Self {
            inode,
            file_off: 0,
            flag: FileFlag::Init,
        }
    }
}
pub enum FileFlag {
    Init,
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



