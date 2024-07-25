use core::{mem::size_of, slice};

use os_in_rust_common::{bitmap::BitMap, constants, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, printkln, racy_cell::RacyCell, utils, ASSERT, MY_PANIC};

use crate::{device::ata::{Disk, Partition}, memory};

use super::{constant, dir::Dir, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

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
    root_dir: Option<RacyCell<Dir<'static>>>,

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

    /**
     * 加载根目录。
     */
    #[inline(never)]
    pub fn load_root_dir(&mut self) {
        let root_inode = self.inode_open(self.super_block.root_inode_no);
        self.root_dir = Option::Some(RacyCell::new(Dir::new(root_inode)));
    }

    #[inline(never)]
    pub fn get_root_dir(&self) -> &'static mut Dir<'static> {
        let file_system = self::get_filesystem();
        let root_dir = file_system.root_dir.as_mut();
        ASSERT!(root_dir.is_some());
        let root_dir = root_dir.unwrap();
        unsafe { root_dir.get_mut() }
    }
    /**
     * 从该文件系统中，根据inode_no打开一个Inode
     */
    #[inline(never)]
    pub fn inode_open(&mut self, i_no: InodeNo) -> &'static mut OpenedInode {
        // 现在已打开的列表中找到这个inode
        for inode_tag in self.open_inodes.iter() {
            let inode = OpenedInode::parse_by_tag(inode_tag);
            if inode.i_no == i_no {
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
     * 根据inode号从硬盘中加载inode
     * 入参：
     *   - i_no: 要加载的inode号
     * 返回值：
     *   - Inode： 加载到的inode
     */
    fn load_inode(&self, i_no: InodeNo) -> Inode {
        let inode_location = self.locate_inode(i_no);
        let disk = unsafe { &mut *self.base_part.from_disk };
        let byte_cnt = inode_location.sec_cnt * constants::DISK_SECTOR_SIZE;
        let inode_buf = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(byte_cnt) as *mut u8, byte_cnt) };
        // 从硬盘中读取扇区
        disk.read_sectors(inode_location.lba, inode_location.sec_cnt, inode_buf);

        let mut target_inode = Inode::empty();
        // 根据字节偏移量，找到这个inode数据
        target_inode = unsafe { *(inode_buf[inode_location.bytes_off .. ].as_ptr() as usize as *const Inode) };
        memory::sys_free(inode_buf.as_ptr() as usize);

        target_inode
    }


    /**
     * 根据inode号计算出，该inode所处硬盘的哪个位置
     */
    #[inline(never)]
    pub fn locate_inode(&self, i_no: InodeNo) -> InodeLocation {
        if u32::from(i_no) >  constant::MAX_FILE_PER_FS {
            MY_PANIC!("failed to locate inode({:?}). exceed maximum({})", i_no, constant::MAX_FILE_PER_FS);
        }
        // inode所在相对inode数组，开始的字节偏移量
        let i_idx_start = usize::from(i_no) * size_of::<Inode>();
        // 换算成扇区偏移数
        let sec_start = i_idx_start / constants::DISK_SECTOR_SIZE;

        // inode所在相对inode数组，结束的字节偏移量
        let i_idx_end = (usize::from(i_no) + 1) as usize * size_of::<Inode>();
        // inode结束的偏移量，换算成扇区偏移数
        let sec_end: usize = utils::div_ceil(i_idx_end as u32, constants::DISK_SECTOR_SIZE  as u32).try_into().unwrap();
        InodeLocation {
            lba: self.super_block.inode_table_lba.add(sec_start.try_into().unwrap()),
            bytes_off: (i_idx_start % constants::DISK_SECTOR_SIZE).try_into().unwrap(),
            sec_cnt: sec_end.max(sec_start + 1) - sec_start,
        }
    }

    /**
     * 申请一个间接块
     *  - 如果间接块已经存在，那也不用申请
     */
    #[inline(never)]
    pub fn apply_indirect_data_block(&mut self, opened_inode: &mut OpenedInode) {
        // 数组最后一个元素，是间接块的LBA地址。这个块里面，是很多的LBA地址
        let indirect_lba = *unsafe { opened_inode.indirect_block_lba.get_mut() };
        if !indirect_lba.is_empty() {
            return;
        }
        opened_inode.indirect_block_lba = RacyCell::new(self.data_block_pool.apply_block(1));
    }

    /**
     * 加载间接块的数据
     *  - 如果不存在间接块，那也不用加载
     */
    #[inline(never)]
    pub fn load_indirect_data_block(&mut self, opened_inode: &mut OpenedInode) {

        // 数组最后一个元素，是间接块的LBA地址。这个块里面，是很多的LBA地址
        let indirect_lba = *unsafe { opened_inode.indirect_block_lba.get_mut() };
        let disk = unsafe { &mut *self.base_part.from_disk };

        // 如果间接块是空的，需要申请一个块
        if indirect_lba.is_empty() {
            return;
        }

        // 数组的剩下元素，转成一个u8数组
        let left_unfilled_lba = opened_inode.get_indirect_data_blocks();
        let buf = unsafe { slice::from_raw_parts_mut(left_unfilled_lba.as_mut_ptr() as *mut u8, left_unfilled_lba.len() * (size_of::<LbaAddr>() / size_of::<u8>())) };
        // 读取硬盘。把数据写入到数组里。最终也是写入到缓存里了
        disk.read_sectors(indirect_lba, 1, buf)
    }

    /**
     * 把内存中的inode数据同步到硬盘中
     *  1. 同步inode自身的数据（包含直接块的地址）
     *  2. 同步inode间接块的数据
     */
    #[inline(never)]
    pub fn sync_inode(&mut self, opened_inode: &mut OpenedInode) {
        let disk = unsafe { &mut *self.base_part.from_disk };

        /*****1. 同步inode自身（包含直接块的地址）*************/
        // 当前inode，所处磁盘的位置
        let i_location = self.locate_inode(opened_inode.i_no);

        // 申请缓冲区大小 = 读取到该inode需要多少个扇区
        let buf_size = constants::DISK_SECTOR_SIZE * i_location.sec_cnt;
        let buff_addr = memory::sys_malloc(buf_size);
        let buf = unsafe { slice::from_raw_parts_mut(buff_addr as *mut u8, buf_size) };

        // 读取出inode所在的扇区
        disk.read_sectors(i_location.lba, i_location.sec_cnt, buf);

        // 硬盘中的inode结构
        let inode_from_disk = unsafe { &mut *(buf.as_mut_ptr().add(i_location.bytes_off) as *mut Inode) };
        // 把内存中的inode结构，复制到硬盘的inode结构中
        inode_from_disk.from(opened_inode);

        // 把inode写回到硬盘中
        disk.write_sector(buf, i_location.lba, i_location.sec_cnt.try_into().unwrap());


        /*****2. 处理inode的间接块***************/
        let indirect_block_lba = unsafe { opened_inode.indirect_block_lba.get_mut() };
        // 没有间接块地址，也不用同步了
        if indirect_block_lba.is_empty() {
            return;
        }

        // 间接块里面全部都是LBA地址
        let indirect_block_sec_lba = unsafe { slice::from_raw_parts_mut(buff_addr as *mut LbaAddr, constants::DISK_SECTOR_SIZE * 2) };
        // 用内存的数据，覆盖硬盘的数据
        indirect_block_sec_lba.copy_from_slice(opened_inode.get_indirect_data_blocks_ref());
        // 写回到硬盘中
        disk.write_sector(buf, *indirect_block_lba, 1);
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
