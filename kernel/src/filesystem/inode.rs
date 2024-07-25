use core::{mem::size_of, ptr};

use os_in_rust_common::{constants, domain::{InodeNo, LbaAddr}, elem2entry, linked_list::LinkedNode};
use os_in_rust_common::racy_cell::RacyCell;

use crate::{memory, sync::Lock, thread};

use super::constant;


/**
 * inode的元信息。存储在硬盘的物理结构
 */
#[derive(Debug)]
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Inode {
    /**
     * inode编号
     */
    pub i_no: InodeNo,
    /**
     * 当前inode占用的空间大小。单位：字节
     * inode是文件，那么i_size是文件大小
     * inode是目录，那么i_size是该目录下所有目录项的大小（不递归）
     */
    pub i_size: u32,

    /**
     * 该inode数据扇区所在的LBA地址。
     */
    pub direct_sectors: [LbaAddr; constant::INODE_DIRECT_DATA_SECS],

    /**
     * 该inode数据扇区所在的LBA地址。
     */
    pub indirect_sector: LbaAddr,
}

impl Inode {
    pub fn empty() -> Self {
        Self {
            i_no: InodeNo::new(0),
            i_size: 0,
            direct_sectors: [LbaAddr::empty(); constant::INODE_DIRECT_DATA_SECS],
            indirect_sector: LbaAddr::empty(),
        }
    }
    pub fn new(i_no: InodeNo) -> Self {
        Self {
            i_no,
            i_size: 0,
            direct_sectors: [LbaAddr::empty(); constant::INODE_DIRECT_DATA_SECS],
            indirect_sector: LbaAddr::empty(),
        }
    }

    pub fn from(&mut self, opened_inode: &OpenedInode) {
        self.i_no = opened_inode.i_no;
        self.i_size = opened_inode.i_size;
        self.direct_sectors.copy_from_slice(opened_inode.get_direct_data_blocks_ref());
        self.indirect_sector = unsafe {*opened_inode.indirect_block_lba.get_mut()};
    }

}

/**
 * 加载的Inode的逻辑结构。内存中的结构
 */
#[derive(Debug)]
pub struct OpenedInode {
    /**
     * inode编号
     */
    pub i_no: InodeNo,
    /**
     * 当前inode占用的空间大小。单位：字节
     * inode是文件，那么i_size是文件大小
     * inode是目录，那么i_size是该目录下所有目录项的大小（不递归）
     */
    pub i_size: u32,

    /**
     * 该inode打开的次数
     */
    pub open_cnts: u32,
    /**
     * 写入拒绝（互斥）
     */
    write_deny: bool,
    /**
     * 标签
     */
    pub tag: LinkedNode,
    /**
     * 锁
     */
    lock: Lock, 

    /**
     * 数据块的缓存。每个元素是一个LBA地址
     * 不包括间接块（因为间接块没有存放数据）
     */
    data_block_list: [LbaAddr; constant::INODE_DIRECT_DATA_SECS + (constant::INODE_INDIRECT_DATA_SECS * constants::DISK_SECTOR_SIZE) / size_of::<LbaAddr>()],
    /**
     * 间接块的地址（这个块内，就是很多的间接数据块的LBA地址）
     */
    pub indirect_block_lba: RacyCell<LbaAddr>,
}

impl OpenedInode {
    pub fn new(base_inode: Inode) -> Self {
        let mut inode = Self {
            i_no: base_inode.i_no,
            i_size: base_inode.i_size,
            open_cnts: 1, // 创建出来就是打开了一次
            write_deny: false,
            tag: LinkedNode::new(),
            lock: Lock::new(),
            indirect_block_lba: RacyCell::new(base_inode.indirect_sector),
            data_block_list: [LbaAddr::empty(); constant::INODE_DIRECT_DATA_SECS + (constant::INODE_INDIRECT_DATA_SECS * constants::DISK_SECTOR_SIZE) / size_of::<LbaAddr>()],
        };
        // 把硬盘中的该inode数据区，复制到缓冲区中
        inode.data_block_list[0..base_inode.direct_sectors.len()].copy_from_slice(&base_inode.direct_sectors);
        inode
    }
    pub fn parse_by_tag(tag: *mut LinkedNode) -> &'static mut OpenedInode {
        unsafe { &mut *elem2entry!(OpenedInode, tag, tag) }
    }


    pub fn get_data_blocks_ref(&self) -> &[LbaAddr] {
        &self.data_block_list
    }

    pub fn get_data_blocks(&mut self) -> &mut [LbaAddr] {
        &mut self.data_block_list
    }

    /**
     * 得到直接数据块地址（存放在inode中）
     */
    pub fn get_direct_data_blocks(&mut self) -> &mut[LbaAddr] {
        &mut self.data_block_list[0 .. constant::INODE_DIRECT_DATA_SECS]
    }

    pub fn get_direct_data_blocks_ref(&self) -> &[LbaAddr] {
        &self.data_block_list[0 .. constant::INODE_DIRECT_DATA_SECS]
    }

    /**
     * 得到间接数据块的地址（存放在inode的间接块内）
     */
    pub fn get_indirect_data_blocks(&mut self) -> &mut[LbaAddr] {
        &mut self.data_block_list[constant::INODE_DIRECT_DATA_SECS .. ]
    }

    pub fn get_indirect_data_blocks_ref(&self) -> &[LbaAddr] {
        &self.data_block_list[constant::INODE_DIRECT_DATA_SECS.. ]
    }

    /**
     * 关闭某一个打开了的Inode
     */
    pub fn inode_close(&mut self) {
        self.lock.lock();
        self.open_cnts -= 1;
        if self.open_cnts == 0 {
            let cur_task = &mut thread::current_thread().task_struct;
            let pgdir_bak = cur_task.pgdir;
            cur_task.pgdir = ptr::null_mut();
            memory::sys_free(self as *const _ as usize);
            cur_task.pgdir = pgdir_bak;
            return;
        }
        self.lock.unlock();
    }
}

/**
 * inode所在磁盘的位置
 */
pub struct InodeLocation {
    /**
     * 该inode所在扇区的LBA地址
     */
    pub lba: LbaAddr,
    /**
     * 该inode所在该扇区内的偏移量（单位字节）
     */
    pub bytes_off: usize,
    /**
     * 该inode是否跨2个扇区（如果跨两个扇区，那么要想读取该inode需要读取两个扇区）
     */
    pub sec_cnt: usize,
}

impl InodeLocation {
    #[inline(never)]
    pub fn new(lba: LbaAddr, bytes_off: usize, sec_cnt: usize) -> Self {
        Self {
            lba: lba,
            bytes_off: bytes_off,
            sec_cnt: sec_cnt,
        }
    }
}
