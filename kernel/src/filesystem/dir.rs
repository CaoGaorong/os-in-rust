use core::{fmt::Display, mem::{self, size_of}, ptr, slice};

use os_in_rust_common::{bitmap::BitMap, constants, cstr_write, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, printkln, racy_cell::RacyCell, ASSERT, MY_PANIC};

use crate::{device::ata::Partition, memory, thread};

use super::{constant, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

/**
 * 文件系统中的目录的结构以及操作
 */

/**
 * 根目录
 */
static ROOT_DIR: RacyCell<Dir> = RacyCell::new(Dir::empty());


/**
 * inode池。逻辑结构
 */
pub struct InodePool {
    /**
     * 池子位图所在硬盘自身的LBA地址
     */
    self_bitmap_lba: LbaAddr,
    /**
     * 该inode池，指向的起始inode号
     */
    start_ino: InodeNo,
    /**
     * inode池的位图
     */
    inode_bitmap: BitMap,
}

impl InodePool {
    pub const fn new(self_lba: LbaAddr, start_ino: InodeNo, inode_bits: &mut [u8]) -> Self {
        Self {
            self_bitmap_lba: self_lba,
            start_ino,
            inode_bitmap: BitMap::new(inode_bits),
        }
    }

    /**
     * 从inode池中申请一个inode
     */
    pub fn apply_inode(&mut self, inodes: usize) -> InodeNo {
        let bit_res = self.inode_bitmap.apply_bits(inodes);
        ASSERT!(bit_res.is_ok());
        let bit_off = bit_res.unwrap();
        // 申请到的inode地址 = inode起始号 + 申请的第x个inode
        self.start_ino.add(bit_off)
    }


    /**
     * 定位inode号为ino时，对应inode位图中的该位 所在的硬盘LBA地址 和 该位图扇区的数据
     * ret:
     *  LbaAddr: 位图该位所在硬盘的LBA地址
     *  &[u8]: 该位图该位所在硬盘扇区数据
     */
    pub fn locate(&self, ino: InodeNo) -> (LbaAddr, &[u8]) {
        ASSERT!(u32::from(ino) >= u32::from(self.start_ino));
        // 该inode，所在位图内偏移量（单位：位）
        let inode_off = ino - self.start_ino;

        // 该inode所在扇区，相对于起始扇区的偏移量（单位：扇区）
        let sec_off = usize::from(inode_off) / 8 / constants::DISK_SECTOR_SIZE;
        // 该扇区的绝对LBA地址
        let abs_sec_idx = self.self_bitmap_lba + LbaAddr::new(sec_off.try_into().unwrap());

        let bitmap_offset = unsafe { self.inode_bitmap.map_ptr.add(sec_off / constants::DISK_SECTOR_SIZE) };
        // 把 inode bitmap 数据区转成数组
        let sec_data = unsafe { slice::from_raw_parts(bitmap_offset, constants::DISK_SECTOR_SIZE) };

        (abs_sec_idx, sec_data)
    }
}


/**
 * 数据块池。逻辑结构
 */
pub struct DataBlockPool {
    /**
     * 池子中数据块位图  自身 所在硬盘的LBA地址
     */
    self_bitmap_lba: LbaAddr,
    /**
     * 池子起始块的LBA地址
     */
    block_start_lba: LbaAddr,
    /**
     * 池子中的块位图 结构
     */
    block_bitmap: BitMap, 
}

impl DataBlockPool {
    pub fn new(self_lba:  LbaAddr, block_start_lba: LbaAddr, block_bits: &mut [u8]) -> Self {
        Self {
            self_bitmap_lba: self_lba,
            block_start_lba: block_start_lba,
            block_bitmap: BitMap::new(block_bits),
        }
    }

    /**
     * 申请blocks个数据块，得到数据块的地址
     */
    pub fn apply_block(&mut self, blocks: usize) -> LbaAddr {
        let res = self.block_bitmap.apply_bits(blocks);
        ASSERT!(res.is_ok());
        let bit_off = res.unwrap();
        // 申请到的块LBA地址 = 起始块LBA + 申请到的第bit_off位
        self.block_start_lba.add(bit_off.try_into().unwrap())
    }

    /**
     * 定位该数据块对应的位，所在位图的信息。
     * ret:
     *  LbaAddr: 位图该位所在硬盘的LBA地址
     *  &[u8]: 该位图该位所在硬盘扇区数据
     */
    pub fn locate_bitmap(&self, block_lba: LbaAddr) -> (LbaAddr, &[u8]) {
        // 这个数据块，在所有数据块的偏移。也就是位图中这个位所在位图的位偏移量
        let bit_off = block_lba - self.block_start_lba;
        
        // 第bit_idx，是inode位图所在扇区的的第bix_in_sec个扇区
        let bit_in_sec = usize::from(bit_off) / 8 / constants::DISK_SECTOR_SIZE;

        // 把 inode bitmap 中，该bit所在地址为起始地址。
        let bitmap_offset = unsafe { self.block_bitmap.map_ptr.add(bit_in_sec / constants::DISK_SECTOR_SIZE) };
        // 把 inode bitmap 数据区转成数组
        let sec_data = unsafe { slice::from_raw_parts(bitmap_offset, constants::DISK_SECTOR_SIZE) };

        (self.self_bitmap_lba + bit_off, sec_data)
    }
    
}

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
     * inode池子
     */
    pub inode_pool: InodePool, 

    /**
     * 空闲块池子
     */
    pub data_block_pool: DataBlockPool, 

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
            inode_pool: InodePool::new(super_block.inode_bitmap_lba, InodeNo::new(0), inode_bits),
            data_block_pool: DataBlockPool::new(super_block.block_bitmap_lba, super_block.data_lba_start, block_bits),
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
    pub fn inode_open(&mut self, i_no: InodeNo) -> &mut OpenedInode {
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
    fn load_inode(&self, i_no: InodeNo) -> Inode {
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
        disk.read_sectors(inode_location.lba, sec_cnt, inode_buf);

        let mut target_inode = Inode::empty();
        // 根据字节偏移量，找到这个inode数据
        target_inode = unsafe { *(inode_buf[inode_location.bytes_off .. ].as_ptr() as usize as *const Inode) };
        memory::sys_free(inode_buf.as_ptr() as usize);

        target_inode
    }

    /**
     * 根据inode号计算出，该inode所处硬盘的哪个位置
     */
    fn locate_inode(&self, i_no: InodeNo) -> InodeLocation {
        if u32::from(i_no) >  constant::MAX_FILE_PER_FS{
            MY_PANIC!("failed to locate inode({:?}). exceed maximum({})", i_no, constant::MAX_FILE_PER_FS);
        }
        // inode所在相对inode数组，开始的字节偏移量
        let i_idx_start = usize::from(i_no) * size_of::<Inode>();
        // inode所在相对inode数组，结束的字节偏移量
        let i_idx_end = (usize::from(i_no) + 1) as usize * size_of::<Inode>();
        
        let cross_secs: bool;
        // 如果开始的字节地址 跟 结束的字节地址，不是同一个扇区，那么就是跨扇区了
        if (i_idx_start / constants::DISK_SECTOR_SIZE) !=(i_idx_end / constants::DISK_SECTOR_SIZE) {
            cross_secs = true;
        } else {
            cross_secs = false;
        }
        
        InodeLocation::new(self.super_block.inode_table_lba, i_idx_start, cross_secs)
    }

    /**
     * 把目录项dir_entry放入到parent目录中。并且保存到硬盘
     */
    pub fn sync_dir_entry(&mut self, parent: &Dir, dir_entry: &DirEntry) {
        // 从块位图中，申请1位。得到该位的下标
        let block_idx = self.data_block_pool.apply_block(1);
        // 把这一位同步到快位图的硬盘中
        self.sync_block_pool(block_idx);

        // 找到当前节点的数据区域，然后把目录项放入数据区


    }
    
    /**
     * ino号inode所在的inode位图同步到硬盘
     */
    pub fn sync_inode_pool(&mut self, ino: InodeNo) {
        // 定位这个inode，所在扇区的LBA地址 和 扇区数据
        let (lba, bit_buf) = self.inode_pool.locate(ino);
        
        let disk = unsafe { &mut *self.base_part.from_disk };
        // 把inode bitmap写入到硬盘中
        disk.write_sector(bit_buf, lba, 1);
    }

    /**
     * 空闲块为block_lba所在的块位图，同步到硬盘
     */
    pub fn sync_block_pool(&mut self, block_lba: LbaAddr) {
        // 定位到这个数据块，所在的位图，
        let (lba, bitmap_buf) = self.data_block_pool.locate_bitmap(block_lba);

        let disk = unsafe { &mut *self.base_part.from_disk };
        
        // 把inode bitmap写入到硬盘中
        disk.write_sector(bitmap_buf, lba, 1);
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
    inode: Option<&'static OpenedInode>,

}

impl Dir {
    pub const fn empty () -> Self {
        Self {
            inode: Option::None
        }
    }

}
/**
 * 目录项的结构。物理结构，保存到硬盘中
 */
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

    /**
     * 把当前的目录项写入parent_dir目录内，保存到硬盘中
     */
    pub fn write_to_disk(&self, parent_dir: &Dir) {
        let parent_inode = parent_dir.inode;
        ASSERT!(parent_inode.is_some());
        let parent_inode = parent_inode.unwrap();

        // TODO
    }
}