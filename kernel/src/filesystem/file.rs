use core::mem::size_of;

use os_in_rust_common::{constants, racy_cell::RacyCell, ASSERT};

use crate::{filesystem::dir::{DirEntry, FileType}, memory};

use super::{constant, dir::{Dir, MountedPartition}, inode::{Inode, OpenedInode}};

/**
 * 整个系统打开的文件
 */
const REPEAT_FILE:Option<OpenedFile> = Option::None;
static GLOBAL_FILE_TABLE: RacyCell<[Option<OpenedFile>; constant::MAX_OPENED_FILE_IN_SYSTEM]> = RacyCell::new([REPEAT_FILE; constant::MAX_OPENED_FILE_IN_SYSTEM]);

/**
 * 从全局的文件表中，找到空位
 */
pub fn get_free_slot_in_global_file_table() -> Option<usize> {
    let global_file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    for (idx, ele) in global_file_table.iter().enumerate() {
        if ele.is_none() {
            return Option::Some(idx);
        }
    }
    return Option::None;
}

/**
 * 填充文件表的第idx项的值为file
 */
pub fn set_file_table(idx: usize, file: OpenedFile) {
    let global_file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    global_file_table[idx] = Option::Some(file);
}

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

/**
 * 在part分区中，parent_dir文件夹下创建一个名为file_name的文件
 */
pub fn create_file(part: &mut MountedPartition, parent_dir: &Dir, file_name: &str) {
    // 从当前分区中，申请1个inode。得到inode号（inode数组的下标）
    let inode_no = part.inode_pool.apply_inode(1);

    // 创建的inode
    let inode: &mut OpenedInode = memory::malloc(size_of::<OpenedInode>());
    *inode = OpenedInode::new(Inode::new(inode_no));


    // 获取文件表
    let file_table_idx = get_free_slot_in_global_file_table();
    ASSERT!(file_table_idx.is_some());
    let file_table_idx = file_table_idx.unwrap();

    // 填充文件表
    set_file_table(file_table_idx, OpenedFile::new(inode));

    let dir_entry = DirEntry::new(inode_no, file_name, FileType::Regular);
    
}



