use core::{fmt::Display, mem::{self, size_of}, ptr, slice};

use os_in_rust_common::{bitmap::BitMap, constants, linked_list::LinkedList, printkln, MY_PANIC};

use crate::{device::ata::Partition, memory, thread};

use super::{constant, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

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

    pub fn inode_close(&mut self, inode: &mut OpenedInode) {
        self.open_inodes.remove(&inode.tag);
        inode.inode_close();
    }
    /**
     * 从该挂载的分区中，根据inode_no打开一个Inode
     */
    pub fn inode_open(&mut self, i_no: u32) -> &mut OpenedInode {
        // 现在已打开的列表中找到这个inode
        for inode_tag in self.open_inodes.iter() {
            let inode = OpenedInode::parse_by_tag(inode_tag);
            if inode.base_inode.i_no == i_no {
                // 打开次数 + 1
                inode.open_cnts += 1;
                return inode;
            }
        }

        // 从堆中申请内存。常驻内存
        let mut opened_inode: &mut OpenedInode = memory::malloc(size_of::<OpenedInode>());

        // 如果已打开列表没有这个inode，那么需要从硬盘中加载
        let inode = self.load_inode(i_no);
        // 把加载的inode，封装为一个打开的结构
        *opened_inode = OpenedInode::new(inode);

        // 添加到打开的列表中
        self.open_inodes.append(&mut opened_inode.tag);
        return opened_inode;
    }

    /**
     * 根据inode号从硬盘中加载
     */
    fn load_inode(&self, i_no: u32) -> Inode {
        let inode_location = self.locate_inode(i_no);
        let disk = unsafe { &mut *self.base_part.from_disk };
        let sec_cnt: usize = if inode_location.cross_secs {
            2
        } else {
            1
        };
        let byte_cnt = sec_cnt * constants::DISK_SECTOR_SIZE;
        let inode_buf = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(byte_cnt) as *mut u8, byte_cnt) };
        // 从硬盘中读取扇区
        disk.read_sectors(inode_location.lba.try_into().unwrap(), sec_cnt, inode_buf);

        let mut target_inode = Inode::empty();
        // 根据字节偏移量，找到这个inode数据
        target_inode = unsafe { *(inode_buf[inode_location.bytes_off .. ].as_ptr() as usize as *const Inode) };
        memory::sys_free(inode_buf.as_ptr() as usize);

        target_inode
    }

    /**
     * 根据inode号计算出，该inode所处硬盘的哪个位置
     */
    fn locate_inode(&self, i_no: u32) -> InodeLocation {
        if i_no >  constant::MAX_FILE_PER_FS{
            MY_PANIC!("failed to locate inode({}). exceed maximum({})", i_no, constant::MAX_FILE_PER_FS);
        }
        // inode所在相对inode数组，开始的字节偏移量
        let i_idx_start = i_no as usize * size_of::<Inode>();
        // inode所在相对inode数组，结束的字节偏移量
        let i_idx_end = (i_no + 1) as usize * size_of::<Inode>();
        
        let cross_secs: bool;
        // 如果开始的字节地址 跟 结束的字节地址，不是同一个扇区，那么就是跨扇区了
        if (i_idx_start / constants::DISK_SECTOR_SIZE) !=(i_idx_end / constants::DISK_SECTOR_SIZE) {
            cross_secs = true;
        } else {
            cross_secs = false;
        }
        
        InodeLocation::new(self.super_block.inode_table_lba, i_idx_start, cross_secs)
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