use core::{fmt::Display, mem::{self, size_of}, ops::Index, ptr, slice};

use os_in_rust_common::{bitmap::BitMap, constants, cstr_write, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, printkln, racy_cell::RacyCell, ASSERT, MY_PANIC, utils};

use crate::{device::ata::{Disk, Partition}, memory, thread};

use super::{constant, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

/**
 * 文件系统中的目录的结构以及操作
 */

/**
 * 根目录
 */
static ROOT_DIR: RacyCell<Dir> = RacyCell::new(Dir::empty());

/**
 * 文件的类型
 */
#[derive(Copy, Clone)]
pub enum FileType {
    /**
     * 普通文件
     */
    Regular,
    /**
     * 目录
     */
    Directory,
    /**
     * 未知
     */
    Unknown,
}
/**
 * 目录的结构。位于内存的逻辑结构
 */
pub struct Dir {
    pub inode: RacyCell<Option<OpenedInode>>,
}

impl Dir {
    pub const fn empty () -> Self {
        Self {
            inode: RacyCell::new(Option::None)
        }
    }

}
/**
 * 目录项的结构。物理结构，保存到硬盘中
 */
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DirEntry {
    /**
     * 该目录项对应的inode编号
     */
    pub i_no: InodeNo, 
    /**
     * 目录项名称
     */
    pub name:  [u8; constant::MAX_FILE_NAME],
    /**
     * 文件类型
     */
    pub file_type: FileType,
}

impl DirEntry {
    pub fn new(i_no: InodeNo, file_name: &str, file_type: FileType) -> Self {
        let mut dir_entry = Self {
            i_no: i_no,
            name: [0; constant::MAX_FILE_NAME],
            file_type: file_type,
        };
        // 写入文件名称
        cstr_write!(&mut dir_entry.name, "{}", file_name);
        dir_entry
    }

    pub fn is_valid(&self) -> bool {
        usize::from(self.i_no) != 0
    }

    /**
     * 把当前的目录项写入parent_dir目录内，保存到硬盘中
     */
    pub fn write_to_disk(&self, parent_dir: &Dir) {
        let parent_inode = unsafe { parent_dir.inode.get_mut() }.as_mut();
        ASSERT!(parent_inode.is_some());
        let parent_inode = parent_inode.unwrap();

        // TODO
    }
}