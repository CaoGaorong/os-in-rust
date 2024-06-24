use core::mem::size_of;

use os_in_rust_common::{constants, utils};

use super::{constant, inode::Inode};

/**
 * 文件系统的超级块
 */

/**
 * 文件系统超级块的结构。物理结构。512个字节
 */
#[derive(Debug)]
#[repr(C, align(512))]
pub struct SuperBlock {
    /**
     * 魔数
     */
    magic: u32,
    /**
     * 本文件系统，起始LBA地址
     */
    lba_start: u32,
    /**
     * 该文件系统中，扇区的数量。也就是本分区中扇区的数量
     */
    sec_cnt: u32,
    
    /**
     * inode节点的数量
     */
    inode_cnt: u32,

    /**
     * 根inode编号
     */
    root_inode_no: usize,
   
    /**
     * inode位图所在扇区的LBA地址
     */
    inode_bitmap_lba: u32,
    /**
     * inode位图占用的扇区数量
     */
    inode_bitmap_secs: u32,

    /**
     * inode数组所在的LBA起始地址
     */
    inode_table_lba: u32,
    /**
     * inode数组，占用的扇区数量
     */
    inode_table_secs: u32,


    /**
     * 空闲块位图，所在扇区的LBA地址
     */
    block_bitmap_lba: u32,
    /**
     * 空闲块位图，占用的扇区数量
     */
    block_bitmap_secs: u32,
 
    /**
     * 数据扇区开始的LBA地址。接在上面元信息的后面
     */
    data_lba_start: u32,
}

impl SuperBlock {
    /**
     * 构建超级块。超级块是文件系统的元数据的元数据。
     * 我们的文件系统数据占据的扇区的结构这样的：
     * | 引导块(1扇区) | 超级块(1扇区) | inode位图(x扇区) | inode数组(y扇区)| 空闲数据块位图(z扇区)  | 数据块
     */
    pub fn new(part_lba: u32, part_secs: u32) -> Self {

        // inode位图所在扇区的起始LBA= 开始LBA + 引导块 + 超级块
        let inode_bitmap_lba = part_lba + 1 + 1;
        // inode位图占用的扇区数量 = inode最大数量 / 一个扇区的位数
        let inode_bitmap_sec = utils::div_ceil(constant::MAX_FILE_PER_FS, constants::DISK_SECTOR_SIZE as u32 * 8) as u32;
        
        let inode_table_lba = inode_bitmap_lba + inode_bitmap_sec;
        // inode数组占用的扇区数量 = inode数量 * inode大小 / 一个扇区的大小
        let inode_table_sec = utils::div_ceil(constant::MAX_FILE_PER_FS * size_of::<Inode>() as u32, constants::DISK_SECTOR_SIZE as u32) as u32;

        let block_bitmap_lba  = inode_table_lba + inode_table_sec;
        // 剩余可用扇区的数量 = 该分区总扇区数量 - 引导块（1扇区） - 超级块（1扇区） - inode位图占扇区数量 - inode数组占扇区数量
        let left_secs = part_secs - 1 - size_of::<SuperBlock>() as u32 - inode_bitmap_sec - inode_table_sec;
        // 空闲块位图 占用的扇区数量 = 剩余扇区 / 每个扇区包含的位数
        let block_bitmap_secs =  utils::div_ceil(left_secs, constants::DISK_SECTOR_SIZE as u32 * 8) as u32;
        // 空闲块位图 占用的扇区数量 = （剩余扇区 - 位图自身占用的扇区数量） / 每个扇区包含的位数
        let block_bitmap_secs = utils::div_ceil((left_secs - block_bitmap_secs), constants::DISK_SECTOR_SIZE as u32 * 8) as u32;


        Self {
            magic: constant::FILESYSTEM_MAGIC,
            lba_start: part_lba, // 分区的起始扇区LBA地址
            sec_cnt: part_secs, // 该分区的扇区数量
            inode_cnt: constant::MAX_FILE_PER_FS,
            root_inode_no: 0, // 根目录的inode号就是0，位于inode数据的首个元素
            // inode位图
            inode_bitmap_lba: inode_bitmap_lba, // inode位图所在扇区的起始LBA
            inode_bitmap_secs: inode_bitmap_sec,// inode位图占用扇区数量
            // inode数组
            inode_table_lba: inode_table_lba, // inode数组所在扇区的起始LBA
            inode_table_secs: inode_table_sec, // inode数组占用扇区的数量
            // 空闲块位图
            block_bitmap_lba: block_bitmap_lba, // 空闲块位图 所在扇区的起始LBA
            block_bitmap_secs: block_bitmap_secs, // 空闲块位图 占用扇区数量
            // 空闲块起始LBA地址，跳过前面的所有块
            data_lba_start: block_bitmap_lba + block_bitmap_secs,
        }
    }
}