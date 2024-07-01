use core::{mem::size_of, ptr, slice};

use os_in_rust_common::{constants, domain::LbaAddr, elem2entry, linked_list::{LinkedList, LinkedNode}};

use crate::{device::ata::Disk, memory, sync::Lock, thread};

use super::{constant, dir::MountedPartition, superblock::SuperBlock};

/**
 * inode的元信息。存储在硬盘的物理结构
 */
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Inode {
    /**
     * inode编号
     */
    pub i_no: u32,
    /**
     * 当前inode占用的空间大小。单位：字节
     * inode是文件，那么i_size是文件大小
     * inode是目录，那么i_size是该目录下所有目录项的大小（不递归）
     */
    pub i_size: u32,

    /**
     * 该inode数据扇区所在的LBA地址。
     * 文件的数据内容分布在不同的扇区。i_sectors[0] = 112。位于第112扇区
     * 12个直接块 + 1个间接块
     */
    pub i_sectors: [LbaAddr; constant::INODE_DATA_SECS],
}

impl Inode {
    pub fn empty() -> Self {
        Self {
            i_no: 0,
            i_size: 0,
            i_sectors: [LbaAddr::empty(); constant::INODE_DATA_SECS],
        }
    }
    pub fn new(i_no: u32) -> Self {
        Self {
            i_no,
            i_size: 0,
            i_sectors: [LbaAddr::empty(); constant::INODE_DATA_SECS],
        }
    }
}

/**
 * 加载的Inode的逻辑结构。内存中的结构
 */
pub struct OpenedInode {
    /**
     * inode
     */
    pub base_inode: Inode,
    
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
     */
    pub data_block_lba: [LbaAddr; constant::INODE_DIRECT_DATA_SECS + (constant::INODE_INDIRECT_DATA_SECS * constants::DISK_SECTOR_SIZE) / size_of::<LbaAddr>()]
}

impl OpenedInode {
    pub fn new(base_inode: Inode) -> Self {
        Self {
            base_inode,
            open_cnts: 1, // 创建出来就是打开了一次
            write_deny: false,
            tag: LinkedNode::new(),
            lock: Lock::new(),
            data_block_lba: [LbaAddr::empty(); constant::INODE_DIRECT_DATA_SECS + (constant::INODE_INDIRECT_DATA_SECS * constants::DISK_SECTOR_SIZE) / size_of::<LbaAddr>()],
        }
    }
    pub fn parse_by_tag(tag: *mut LinkedNode) -> &'static mut OpenedInode {
        unsafe { &mut *elem2entry!(OpenedInode, tag, tag) }
    }

    /**
     * 加载所有数据块的LBA地址
     */
    pub fn load_data_block(&mut self, part: &MountedPartition) {
        for idx in 0 .. constant::INODE_DIRECT_DATA_SECS {
            self.data_block_lba[idx] = self.base_inode.i_sectors[idx];
        }
        // 数组最后一个元素，是间接块的LBA地址。这个块里面，是很多的LBA地址
        let indirect_lba = self.base_inode.i_sectors[constant::INODE_DIRECT_DATA_SECS];
        let disk = unsafe { &mut *part.base_part.from_disk };

        // 数组的剩下元素，转成一个u8数组
        let left_unfilled_lba = &mut self.data_block_lba[constant::INODE_DIRECT_DATA_SECS .. ];
        let buf = unsafe { slice::from_raw_parts_mut(left_unfilled_lba.as_mut_ptr() as *mut u8, left_unfilled_lba.len() * (size_of::<LbaAddr>() / size_of::<u8>())) };
        // 读取硬盘。把数据写入到数组里。最终也是写入到缓存里了
        disk.read_sectors(indirect_lba, 1, buf)
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
    pub cross_secs: bool,
}

impl InodeLocation {
    pub fn new(lba: LbaAddr, bytes_off: usize, cross_secs: bool) -> Self {
        Self {
            lba: lba,
            bytes_off: bytes_off,
            cross_secs: cross_secs,
        }
    }
}
