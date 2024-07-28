use core::slice;

use os_in_rust_common::{bitmap::BitMap, constants, domain::{InodeNo, LbaAddr}, linked_list::{LinkedList, LinkedNodeIterator}, printkln, racy_cell::RacyCell, utils, ASSERT, MY_PANIC};

use crate::device::ata::{Disk, Partition};

use super::{inode::{Inode, OpenedInode}, superblock::SuperBlock};

/**
 * 文件系统。中任何操作都是基于分区的
 */

/**
 * 当前的挂载的分区
 */
static CUR_FILE_SYSTEM: RacyCell<Option<FileSystem>> = RacyCell::new(Option::None);

pub fn set_filesystem(cur_part: FileSystem) {
    *unsafe { CUR_FILE_SYSTEM.get_mut() } = Option::Some(cur_part);
}

#[inline(never)]
pub fn get_filesystem() -> &'static mut FileSystem {
    let fs = unsafe { CUR_FILE_SYSTEM.get_mut() }.as_mut();
    ASSERT!(fs.is_some());
    fs.unwrap()
}

/**
 * 目录的结构。位于内存的逻辑结构
 */
pub struct Dir {
    pub inode: OpenedInode,
}

impl Dir {
    pub fn new(inode: OpenedInode) -> Self {
        Self {
            inode,
        }
    }
    pub fn get_inode_ref(&mut self) -> &mut OpenedInode {
        &mut self.inode
    }
}

/**
 * 文件系统的逻辑结构。
 * 把硬盘中的文件系统静态数据，加载到内存中
 */
pub struct FileSystem {
    /**
     * 挂载中的分区
     */
    pub base_part: &'static Partition,

    /**
     * 该挂载的分区的超级块所在的内存地址
     */
    pub super_block: &'static SuperBlock,

    /**
     * 根目录
     */
    root_dir: Option<RacyCell<Dir>>,

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

unsafe impl Sync for FileSystem {}
unsafe impl Send for FileSystem {}

impl FileSystem {
    /**
     * 创建文件系统。系统首次加载，基于分区
     */
    pub fn new(part: &'static Partition, super_block: &'static SuperBlock, inode_bits: &mut [u8], block_bits: &mut [u8]) -> Self {
        Self {
            base_part: part,
            super_block: super_block,
            root_dir: Option::None,
            inode_pool: InodePool::new(part.from_disk, super_block.inode_bitmap_lba, InodeNo::new(0), inode_bits),
            data_block_pool: DataBlockPool::new(part.from_disk, super_block.block_bitmap_lba, super_block.data_lba_start, block_bits),
            open_inodes: LinkedList::new(),
        }
    }

    pub fn set_root_inode(&mut self, inode: Inode) {
        self.root_dir = Option::Some(RacyCell::new(Dir::new(OpenedInode::new(inode))));
    }

    #[inline(never)]
    pub fn get_root_dir(&self) -> &'static mut Dir {
        let file_system = self::get_filesystem();
        let root_dir = file_system.root_dir.as_mut();
        ASSERT!(root_dir.is_some());
        let root_dir = root_dir.unwrap();
        unsafe { root_dir.get_mut() }
    }

    #[inline(never)]
    pub fn append_inode(&mut self, inode: &mut OpenedInode) {
        self.open_inodes.append(&mut inode.tag);
    }

    #[inline(never)]
    pub fn remove_inode(&mut self, inode: &mut OpenedInode) {
        self.open_inodes.remove(&mut inode.tag)
    }

    #[inline(never)]
    pub fn find_inode(&self, i_no: InodeNo) -> Option<&mut OpenedInode> {
        for inode_tag in self.open_inodes.iter() {
            let exist_inode = OpenedInode::parse_by_tag(inode_tag);
            // 如果找到了
            if exist_inode.i_no == i_no {
                return Option::Some(exist_inode);
            }
        }
        return Option::None;
    }

}
/**
 * inode池。逻辑结构
 */
pub struct InodePool {
    disk: * mut Disk,
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
    pub const fn new(disk: *mut Disk, self_lba: LbaAddr, start_ino: InodeNo, inode_bits: &mut [u8]) -> Self {
        Self {
            disk,
            self_bitmap_lba: self_lba,
            start_ino,
            inode_bitmap: BitMap::new(inode_bits),
        }
    }

    /**
     * 从inode池中申请一个inode
     */
    #[inline(never)]
    pub fn apply_inode(&mut self, inodes: usize) -> InodeNo {
        let bit_res = self.inode_bitmap.apply_bits(inodes);
        ASSERT!(bit_res.is_ok());
        let bit_off = bit_res.unwrap();
        // 设置这位为占用
        self.inode_bitmap.set_bit(bit_off, true);
        // 申请到的inode地址 = inode起始号 + 申请的第x个inode
        let i_no = self.start_ino.add(bit_off);
        // 申请了inode，同步到硬盘
        self.sync_inode_pool(i_no);
        i_no
    }

    /**
     * ino号inode所在的inode位图同步到硬盘
     */
    #[inline(never)]
    pub fn sync_inode_pool(&mut self, ino: InodeNo) {
        let disk = unsafe { &mut *self.disk };
        // 定位这个inode，所在扇区的LBA地址 和 扇区数据
        let (lba, bit_buf) = self.locate(ino);
        // 把inode bitmap写入到硬盘中
        disk.write_sector(bit_buf, lba, 1);
    }

    /**
     * 定位inode号为ino时，对应inode位图中的该位 所在的硬盘LBA地址 和 该位图扇区的数据
     * ret:
     *  LbaAddr: 位图该位所在硬盘的LBA地址
     *  &[u8]: 该位图该位所在硬盘扇区数据
     */
    fn locate(&self, ino: InodeNo) -> (LbaAddr, &[u8]) {
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
    disk: *mut Disk,
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
    pub fn new(disk: *mut Disk, self_lba:  LbaAddr, block_start_lba: LbaAddr, block_bits: &mut [u8]) -> Self {
        Self {
            disk,
            self_bitmap_lba: self_lba,
            block_start_lba: block_start_lba,
            block_bitmap: BitMap::new(block_bits),
        }
    }

    /**
     * 在数据块的池子中，申请一个数据块。（会同步到硬盘）
     */
    #[inline(never)]
    pub fn apply_block(&mut self, blocks: usize) -> LbaAddr {
        // 从块位图申请1位
        let res = self.block_bitmap.apply_bits(blocks);
        if res.is_err() {
            MY_PANIC!("failed to apply block. res:{:?}", res);
        }
        let bit_off = res.unwrap();
        // 把块位图这一位设置为占用
        self.block_bitmap.set_bit(bit_off, true);
        // 申请到的块LBA地址 = 起始块LBA + 申请到的第bit_off位
        let block_lba = self.block_start_lba.add(bit_off.try_into().unwrap());
        // 把申请到的块，同步到硬盘
        self.sync_block_pool(block_lba);
        block_lba
    }

    /**
     * 空闲块为block_lba所在的块位图，同步到硬盘
     */
    #[inline(never)]
    pub fn sync_block_pool(&mut self, block_lba: LbaAddr) {
        let disk = unsafe { &mut *self.disk };
        // 定位到这个数据块，所在的位图，
        let (lba, bitmap_buf) = self.locate_bitmap(block_lba);
        // 把inode bitmap写入到硬盘中
        disk.write_sector(bitmap_buf, lba, 1);
    }

    /**
     * 根据某个数据块的LBA地址，定位对应的位所在位图的区域
     * ret:
     *  LbaAddr: 位图该位所在硬盘的LBA地址
     *  &[u8]: 该位图该位所在硬盘扇区数据
     */
    fn locate_bitmap(&self, block_lba: LbaAddr) -> (LbaAddr, &[u8]) {
        // 位图中「这个位所在位图的位偏移量」
        let bit_off = block_lba - self.block_start_lba;
        
        // 位图中「这个位所在的扇区，相对于位图起始扇区的偏移量」
        let sec_off = usize::from(bit_off) / 8 / constants::DISK_SECTOR_SIZE;

        // 位图所在扇区开头，对应位图的地址
        let bitmap_bit_offset = unsafe { self.block_bitmap.map_ptr.add(sec_off * constants::DISK_SECTOR_SIZE) };
        // 把 inode bitmap 数据区转成数组
        let sec_data = unsafe { slice::from_raw_parts(bitmap_bit_offset, constants::DISK_SECTOR_SIZE) };

        (self.self_bitmap_lba.add(sec_off.try_into().unwrap()), sec_data)
    }
    
}
