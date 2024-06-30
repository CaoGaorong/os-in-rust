use core::ptr;

use os_in_rust_common::{elem2entry, linked_list::{LinkedList, LinkedNode}};

use crate::{memory, sync::Lock, thread};

use super::constant;

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
    pub i_sectors: [Option<u32>; constant::INODE_DATA_SECS],
}

impl Inode {
    pub fn empty() -> Self {
        Self {
            i_no: 0,
            i_size: 0,
            i_sectors: [Option::None; constant::INODE_DATA_SECS],
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
}

impl OpenedInode {
    pub fn new(base_inode: Inode) -> Self {
        Self {
            base_inode,
            open_cnts: 1, // 创建出来就是打开了一次
            write_deny: false,
            tag: LinkedNode::new(),
            lock: Lock::new(),
        }
    }
    pub fn parse_by_tag(tag: *mut LinkedNode) -> &'static mut OpenedInode {
        unsafe { &mut *elem2entry!(OpenedInode, tag, tag) }
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
    pub lba: u32,
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
    pub fn new(lba: u32, bytes_off: usize, cross_secs: bool) -> Self {
        Self {
            lba: lba,
            bytes_off: bytes_off,
            cross_secs: cross_secs,
        }
    }
}
