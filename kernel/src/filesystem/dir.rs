use core::fmt::Display;

use os_in_rust_common::{bitmap::BitMap, linked_list::LinkedList, printkln};

use crate::device::ata::Partition;

use super::{constant, superblock::SuperBlock};

/**
 * 文件系统中的目录的结构以及操作
 */


/**
 * 挂载的分区结构
 */
pub struct MountedPartition {
    /**
     * 挂载中的分区
     */
    pub base_part: &'static Partition,

    /**
     * 该挂载的分区的超级块所在的内存地址
     */
    pub super_block: &'static SuperBlock,

    /**
     * 当前挂载的分区的inode位图
     */
    pub inode_bitmap: BitMap, 

    /**
     * 当前挂载的分区的块位图
     */
    pub block_bitmap: BitMap, 
    
    /**
     * 当前挂载的分区，打开的inode节点队列
     */
    pub open_inodes: LinkedList,
}

impl MountedPartition {
    pub fn new(part: &'static Partition, super_block: &'static SuperBlock, inode_bits: &mut [u8], block_bits: &mut [u8]) -> Self {
        Self {
            base_part: part,
            super_block: super_block,
            inode_bitmap: BitMap::new(inode_bits),
            block_bitmap: BitMap::new(block_bits),
            open_inodes: LinkedList::new(),
        }
    }
}

/**
 * 文件的类型
 */
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

}
/**
 * 目录项的结构。物理结构，保存到硬盘中
 */
#[repr(C, packed)]
pub struct DirEntry {
    /**
     * 该目录项对应的inode编号
     */
    pub i_no: usize, 
    /**
     * 目录项名称
     */
    pub name:  [u8; constant::MAX_FILE_NAME],
    /**
     * 文件类型
     */
    pub file_type: FileType,
}