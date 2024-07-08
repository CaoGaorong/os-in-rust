use core::{fmt::Display, mem::{self, size_of}, ops::Index, ptr, slice};

use os_in_rust_common::{bitmap::BitMap, constants, cstr_write, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, printkln, racy_cell::RacyCell, ASSERT, MY_PANIC, utils};

use crate::{device::ata::{Disk, Partition}, memory, thread};
use crate::filesystem::init::get_filesystem;

use super::{constant, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

/**
 * 文件系统中的目录的结构以及操作
 */

/**
 * 根目录
 */
static ROOT_DIR: RacyCell<Option<Dir>> = RacyCell::new(Option::None);

pub fn get_root_dir() -> Option<&'static mut Dir> {
    let root_dir = unsafe { ROOT_DIR.get_mut() };
    if root_dir.is_none() {
        return Option::None;
    }
    root_dir.as_mut()
}

// #[inline(never)]
pub fn init_root_dir() {
    // printkln!("init root dir");
    let file_system = get_filesystem();
    if file_system.is_none() {
        MY_PANIC!("file system is not loaded");
    }
    let file_system = file_system.unwrap();
    // 打开根目录
    let root_inode = file_system.inode_open(file_system.super_block.root_inode_no);
    let root_dir = unsafe { ROOT_DIR.get_mut() };
    *root_dir = Option::Some(Dir::new(root_inode));
}

/**
 * 文件的类型
 */
#[repr(C)]
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
    pub inode: &'static mut OpenedInode,
}

impl Dir {
    pub fn new(inode: &'static mut OpenedInode) -> Self {
        Self {
            inode,
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
}