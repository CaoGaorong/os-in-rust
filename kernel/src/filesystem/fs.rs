use core::{mem::{self, size_of}, slice};

use os_in_rust_common::{bitmap::BitMap, constants, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, racy_cell::RacyCell, utils, ASSERT, MY_PANIC};

use crate::{device::ata::{Disk, Partition}, memory};

use super::{constant, dir::{Dir, DirEntry}, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

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

pub fn get_filesystem() -> Option<&'static mut FileSystem> {
    unsafe { CUR_FILE_SYSTEM.get_mut() }.as_mut()
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
    pub fn load_root_dir(&mut self) {
        let root_inode = self.inode_open(self.super_block.root_inode_no);
        self.root_dir = Option::Some(RacyCell::new(Dir::new(root_inode)));
    }

    #[inline(never)]
    pub fn get_root_dir(&self) -> &'static mut Dir<'static> {
        let file_system = self::get_filesystem();
        if file_system.is_none() {
            MY_PANIC!("file system is not loaded");
        }
        let file_system =  file_system.unwrap();
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
     * 关闭一个inode
     */
    pub fn inode_close(&mut self, inode: &mut OpenedInode) {
        self.open_inodes.remove(&inode.tag);
        inode.inode_close();
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
     * 把一个inode写入到硬盘中
     * 入参：
     *   - inode: 要同步的inode
     */
    pub fn sync_inode(&self, inode: &Inode) {
        // 先找到这个inode所在硬盘的位置
        let inode_location = self.locate_inode(inode.i_no);
        let disk = unsafe { &mut *self.base_part.from_disk };
        let byte_cnt = inode_location.sec_cnt * constants::DISK_SECTOR_SIZE;
        let byte_buf = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(byte_cnt) as *mut u8, byte_cnt) };
        // 从硬盘中读取扇区，里面是inode数组
        disk.read_sectors(inode_location.lba, inode_location.sec_cnt, byte_buf);

        // 把要同步的inode，写入到inode数组中
        let inode_buf = unsafe { slice::from_raw_parts_mut(byte_buf.as_mut_ptr() as *mut Inode, byte_cnt / size_of::<Inode>()) };
        inode_buf[usize::from(inode.i_no)] = *inode;

        // 把inode数组，写入到硬盘中
        disk.write_sector(byte_buf, inode_location.lba, inode_location.sec_cnt);

        memory::sys_free(byte_buf.as_ptr() as usize);
    }

    /**
     * 根据inode号计算出，该inode所处硬盘的哪个位置
     */
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
            sec_cnt: (sec_end - sec_start).max(1).try_into().unwrap(),
        }
    }


    /**
     * 已知多个数据块data_blocks（一个数据块的内容里面都是多个目录项），找出可以可用的目录项的位置
     * 入参：
     *    - data_blocks 数据扇区信息
     *    - buf 缓冲区，一个扇区大小，存放间接块内的目录项
     *    - disk 操作的硬盘
     * 返回值：
     *    - Option<(&'a mut LbaAddr, &'b mut DirEntry)> 是否找到空闲目录项(该目录项所在扇区的LBA地址，该目录项在buf参数中的地址)
     *  比如buf[512]，发现第100项可用，那么返回值是 &buf[100]
     */
    fn find_available_entry<'a, 'b>(data_blocks: &'a mut [LbaAddr], dir_buf: &'b mut[DirEntry], disk: &mut Disk) -> Option<(&'a mut LbaAddr, &'b mut DirEntry)> {
        let u8buf = unsafe { slice::from_raw_parts_mut(dir_buf.as_mut_ptr() as *mut u8, dir_buf.len() * size_of::<DirEntry>()) };
        // 清空缓冲区
        unsafe { u8buf.as_mut_ptr().write_bytes(0, u8buf.len()); }
        // 先找空的直接块
        let empty_dix = data_blocks
            .iter()
            .enumerate()
            // 找到为空的数据块
            .find(|(idx, &lba)| lba.is_empty())
            .map(|(idx, &lba)| idx);

        // 如果没有空的直接块
        if empty_dix.is_none() {
            // 看看这个最后一个块，有没有空位
            disk.read_sectors(data_blocks[data_blocks.len() - 1], 1, u8buf);
            return dir_buf.iter().enumerate()
                .find(|(idx, &entry)| entry.is_empty())
                .map(|(idx, &entry)| idx)
                // 有空目录项。那么很好，就是这里了。也不用开辟新数据块
                .map(|idx | (&mut data_blocks[data_blocks.len() - 1], &mut dir_buf[idx]))
        }
        // 如果第0项就是空的，那么就返回0了
        if empty_dix.unwrap() == 0 {
            return Option::Some((&mut data_blocks[empty_dix.unwrap()], &mut dir_buf[0]));
        }

        // 如果有空的数据块，那么往前找一个,有没有空位
        disk.read_sectors(data_blocks[empty_dix.unwrap() - 1], 1, u8buf);
        let result = dir_buf.iter().enumerate()
            .find(|(idx, &entry)| entry.is_empty())
            .map(|(idx, &entry)| idx);

        // 有空目录项。那么很好，就是这里了。也不用开辟新数据块
        if result.is_some() {
            Option::Some((&mut data_blocks[empty_dix.unwrap() - 1], &mut dir_buf[result.unwrap()]))
        } else {
            return Option::Some((&mut data_blocks[empty_dix.unwrap()], &mut dir_buf[0]));
        }
    }

    /**
     * 把目录项dir_entry放入到parent目录中。并且保存到硬盘
     *  - 目录项存放在目录inode的数据扇区中
     *  - 先遍历数据扇区，找到空闲可以存放目录项的地方，然后放进去
     */
    #[inline(never)]
    pub fn sync_dir_entry(&mut self, parent: &mut Dir, dir_entry: &DirEntry) {

        let disk = unsafe { &mut *self.base_part.from_disk };

        // 申请内存，搞一个缓冲区
        let buff_addr = memory::sys_malloc(constants::DISK_SECTOR_SIZE);
        let buf = unsafe { slice::from_raw_parts_mut(buff_addr as *mut u8, constants::DISK_SECTOR_SIZE) };
        // 缓冲区格式转成 目录项
        let dir_list = unsafe { slice::from_raw_parts_mut(buff_addr as *mut DirEntry, utils::div_ceil(constants::DISK_SECTOR_SIZE as u32, size_of::<DirEntry>() as u32) as usize ) };


        // 在直接块中，找是否有空闲目录项
        let direct_find = Self::find_available_entry(parent.inode.get_direct_data_blocks(), dir_list, disk);
        // 我们的目录项所在的数据块，位于当前数据块列表的下标
        let block_for_entry = if direct_find.is_some() {
            let (block_lba, entry_addr) = direct_find.unwrap();
            *entry_addr = *dir_entry;
            block_lba
        // 再找间接块
        } else {
            // 先把间接块的数据加载出来（如果间接块不存在，会自动创建）
            parent.inode.load_data_block(self);

            // 在间接块中，找是否有空闲目录项
            let indirect_find = Self::find_available_entry(parent.inode.get_indirect_data_blocks(), dir_list, disk);
            ASSERT!(indirect_find.is_some());
            let (block_lba, entry_addr) = indirect_find.unwrap();
            *entry_addr = *dir_entry;
            block_lba
        };

        // 找到的数据块LBA是空的
        if block_for_entry.is_empty() {
            // 申请一个数据块
            *block_for_entry = self.data_block_pool.apply_block(1);
        }

        // 写入 目录项 到硬盘中
        disk.write_sector(buf, *block_for_entry, 1);

        // 增加当前文件的大小
        parent.inode.i_size += size_of::<DirEntry>() as u32;
        // 如果是直接块找到空闲目录项，那么需要同步inode（直接块的地址放在inode的i_sectors字段中）
        parent.inode.sync_inode(self);

        // 释放缓冲区的空间
        memory::sys_free(buff_addr);
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
    pub fn apply_block(&mut self, blocks: usize) -> LbaAddr {
        // 从块位图申请1位
        let res = self.block_bitmap.apply_bits(blocks);
        ASSERT!(res.is_ok());
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
    pub fn sync_block_pool(&mut self, block_lba: LbaAddr) {
        let disk = unsafe { &mut *self.disk };
        // 定位到这个数据块，所在的位图，
        let (lba, bitmap_buf) = self.locate_bitmap(block_lba);
        // 把inode bitmap写入到硬盘中
        disk.write_sector(bitmap_buf, lba, 1);
    }

    /**
     * 定位该数据块对应的位，所在位图的信息。
     * ret:
     *  LbaAddr: 位图该位所在硬盘的LBA地址
     *  &[u8]: 该位图该位所在硬盘扇区数据
     */
    fn locate_bitmap(&self, block_lba: LbaAddr) -> (LbaAddr, &[u8]) {
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
