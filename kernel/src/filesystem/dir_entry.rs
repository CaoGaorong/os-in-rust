use core::{mem::size_of, slice};

use os_in_rust_common::{constants, cstr_write, cstring_utils, domain::{InodeNo, LbaAddr}, printkln, utils, ASSERT};

use crate::{device::ata::Disk, memory};

use super::{constant, fs::FileSystem, inode::{Inode, OpenedInode}};


/**
 * 文件的类型
 */
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
#[derive(Copy, Clone)]
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
 * 目录项的结构。物理结构，保存到硬盘中
 */
#[derive(Debug)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DirEntry {
    /**
     * 该目录项对应的inode编号
     */
    pub i_no: InodeNo, 
    /**
     * 目录项名称
     */
    name:  [u8; constant::MAX_FILE_NAME],
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

    #[inline(never)]
    pub fn get_name(&self) -> &str {
        let name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(name.is_some());
        name.unwrap()
    }

    #[inline(never)]
    pub fn is_empty(&self) -> bool {
        let i = usize::from(self.i_no);
        // 这是根目录，非空
        if i == 0 && self.file_type as FileType  == FileType::Directory {
            return false;
        }
        return i == 0;
    }
}


pub fn read_dir_entry<'a, 'b>(fs: &'a mut FileSystem, lba: LbaAddr, buff: &'b mut [u8; constants::DISK_SECTOR_SIZE]) -> &'b mut [DirEntry] {
    let disk = unsafe { &mut *fs.base_part.from_disk };
    disk.read_sectors(lba, 1, buff);
    
    unsafe { core::slice::from_raw_parts_mut(buff.as_mut_ptr() as *mut DirEntry, buff.len() / size_of::<DirEntry>()) }
}

/**
 * 指定目录项的路径，搜索这个目录项
 */
#[inline(never)]
pub fn search_dir_entry(filesystem: &mut FileSystem, file_path: &str) -> Option<(DirEntry, &'static mut OpenedInode)> {
    if file_path.is_empty() {
        return Option::None;
    }

    let root_inode_no = filesystem.get_root_dir().inode.i_no;
    
    // 成功搜索过的路径
    let mut searched_path = "/";
    // 当前的inode，是根目录的inode
    let mut cur_inode = filesystem.inode_open(root_inode_no);
    // 当前的目录项。默认是根目录
    let mut cur_dir_entry = DirEntry::new(root_inode_no, "/", FileType::Directory);

    // 如果就是根目录
    if file_path == "/" {
        return Option::Some((cur_dir_entry, cur_inode));
    }

    // 把要搜索的路径，分隔
    let mut file_entry_split = file_path.split("/");
    while let Option::Some(file_entry_name) = file_entry_split.next() {
        if file_entry_name.is_empty() {
            continue;
        }
        // 根据名称搜索目录项
        let dir_entry = do_search_dir_entry(filesystem, &mut cur_inode, file_entry_name);
        // 如果目录项不存在
        if dir_entry.is_none() {
            // printkln!("entry is none: {}, cur_ino: {:?}", file_entry_name, cur_inode);
            return Option::None;
        }

        let dir_entry = dir_entry.unwrap();
        // 根据inode号，打开
        let opened_inode = filesystem.inode_open(dir_entry.i_no);
        cur_inode = opened_inode;

        // 搜索下一个目录项
        cur_dir_entry = dir_entry;
    }
    // 返回找到的最后的那个目录项
    return Option::Some((cur_dir_entry, cur_inode));
}


/**
 * 查找某个目录dir_inode下，名为entry_name的目录项
 */
#[inline(never)]
pub fn do_search_dir_entry(fs: &mut FileSystem, dir_inode: &mut OpenedInode, entry_name: &str) -> Option<DirEntry> {
    if entry_name.is_empty() {
        return Option::None;
    }
    // 如果直接块都满了，那么就需要加载间接块
    if dir_inode.get_direct_data_blocks_ref().iter().all(|block| !block.is_empty()) {
        fs.load_indirect_data_block(dir_inode);
    }

    // 取出所有的数据块
    let data_blocks = dir_inode.get_data_blocks_ref();

    let disk = unsafe { &mut *fs.base_part.from_disk };
    
    // 开辟缓冲区
    let buff_addr = memory::sys_malloc(constants::DISK_SECTOR_SIZE);
    let buff_u8 = unsafe { slice::from_raw_parts_mut(buff_addr as *mut u8, constants::DISK_SECTOR_SIZE) };

    // 遍历所有的数据区，根据名称找目录项
    for block_lba in data_blocks {
        if block_lba.is_empty() {
            continue;
        }
        disk.read_sectors(*block_lba, 1, buff_u8);

        // 读取出的数据，转成页目录项列表
        let dir_entry_list = unsafe { slice::from_raw_parts(buff_addr as *const DirEntry, constants::DISK_SECTOR_SIZE / size_of::<DirEntry>()) };
        let find = dir_entry_list.iter().find(|entry| entry.get_name() == entry_name);
        
        // 找到了，直接返回
        if find.is_some() {
            fs.open_inodes.append(&mut dir_inode.tag);
            return Option::Some(*find.unwrap());
        }
    }
    return Option::None;
}



/**
 * 在filesystem文件系统中，parent_inode目录下创建一个名为dir_name的目录项
 * 在实现上，分成两个步骤：
 *   - 创建文件（inode）以及对应的目录项（文件名称）
 *   - 把这个inode挂到该目录下（把目录项放写入到目录对应的数据区）
 */
#[inline(never)]
pub fn create_dir_entry(fs: &mut FileSystem, parent_inode: &mut OpenedInode, entry_name: &str, file_type: FileType) -> &'static mut OpenedInode {

    /****1. 创建一个目录项 */
    let created_entry_inode = self::do_create_dir_entry(fs, parent_inode, Option::None, entry_name, file_type);

    /***2. 填充内存结构*****/
    fs.open_inodes.append(&mut created_entry_inode.tag);

    // 返回创建好了目录项
    created_entry_inode
}

/**
 * 在parent_inode目录下，创建名为entry_name，并且inode号为entry_inode的目录项。
 */
#[inline(never)]
pub fn do_create_dir_entry(fs: &mut FileSystem, parent_inode: &mut OpenedInode, entry_inode: Option<InodeNo>, entry_name: &str, file_type: FileType) -> &'static mut OpenedInode {
    let (inode_no, opened_inode) = if entry_inode.is_none() {
        /***1. 创建文件的inode。物理结构，同步到硬盘中*****/
        // 从当前分区中，申请1个inode，并且写入硬盘（inode位图）
        let inode_no = fs.inode_pool.apply_inode(1);

        // 创建一个inode
        let inode = Inode::new(inode_no);
        let opened_inode: &mut OpenedInode = memory::malloc(size_of::<OpenedInode>());
        *opened_inode = OpenedInode::new(inode);

        // 把inode写入硬盘（inode列表）
        fs.sync_inode(opened_inode);
        (inode_no, opened_inode)
    } else {
        let inode_no: InodeNo = entry_inode.unwrap();
        let opened_inode = fs.inode_open(inode_no);
        (inode_no, opened_inode)
    };

    /***2. 把这个新文件作为一个目录项，挂到父目录中*****/
    // 创建一个目录项
    let dir_entry = DirEntry::new(inode_no, entry_name, file_type);

    // 把目录项挂到目录并且写入硬盘（inode数据区）
    self::sync_dir_entry(fs, parent_inode, &dir_entry);
    
    opened_inode
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
// #[inline(never)]
fn find_available_entry(data_blocks: &mut [LbaAddr], u8buf: &mut[u8; constants::DISK_SECTOR_SIZE], disk: &mut Disk) -> Option<(usize, usize)> {
    // 再转成DirEntryList
    let dir_buf = unsafe { slice::from_raw_parts(u8buf.as_ptr() as *const DirEntry, u8buf.len() / size_of::<DirEntry>()) };

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
        let last_data_block_idx = data_blocks.len() - 1;
        disk.read_sectors(data_blocks[last_data_block_idx], 1, u8buf);
        return dir_buf.iter().enumerate()
            .find(|(idx, &entry)| entry.is_empty())
            // 有空目录项。那么很好，就是这里了。也不用开辟新数据块
            .map(|(idx, _)| (last_data_block_idx, idx))
    }
    let empty_dix = empty_dix.unwrap();
    // 如果第0项就是空的，那么就返回0了
    if empty_dix == 0 {
        return Option::Some((0, 0));
    }

    // 如果有空的数据块，那么往前找一个,有没有空位
    let previous_idx = empty_dix - 1;
    disk.read_sectors(data_blocks[previous_idx], 1, u8buf);
    
    let previous_find = dir_buf.iter().enumerate()
        .find(|(idx, &entry)| entry.is_empty())
        .map(|(idx, _)| idx);

    // 如果前一个找到了，用前一个
    if previous_find.is_some() {
        Option::Some((previous_idx, previous_find.unwrap()))
    } else {
        // 用新的，需要清空目录项缓冲区
        unsafe { u8buf.as_mut_ptr().write_bytes(0, u8buf.len()); }
        return Option::Some((empty_dix, 0));
    }
}

/**
 * 把目录项dir_entry放入到parent目录中。并且保存到硬盘
 *  - 目录项存放在目录inode的数据扇区中
 *  - 先遍历数据扇区，找到空闲可以存放目录项的地方，然后放进去
 */
#[inline(never)]
pub fn sync_dir_entry(fs: &mut FileSystem, parent_inode: &mut OpenedInode, dir_entry: &DirEntry) {

    let disk = unsafe { &mut *fs.base_part.from_disk };

    // 申请内存，搞一个缓冲区
    let buf: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
    // let buf = unsafe { slice::from_raw_parts_mut(buff_addr as *mut u8, constants::DISK_SECTOR_SIZE) };
    // 缓冲区格式转成 目录项
    let entry_len = utils::div_ceil(constants::DISK_SECTOR_SIZE as u32, size_of::<DirEntry>() as u32) as usize;
    // let entry_len = entry_len - 1;
    // let entry_len = (constants::DISK_SECTOR_SIZE as u32 / size_of::<DirEntry>() as u32) as usize;
    // let entry_len = (constants::DISK_SECTOR_SIZE + size_of::<DirEntry>() - 1) / size_of::<DirEntry>();
    let dir_list = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut DirEntry,  entry_len) };


    // 在直接块中，找是否有空闲目录项
    let direct_find = self::find_available_entry(parent_inode.get_direct_data_blocks(), buf, disk);
    // 我们的目录项所在的数据块，位于当前数据块列表的下标
    let block_idx = if direct_find.is_some() {
        let (block_idx, entry_idx) = direct_find.unwrap();
        dir_list[entry_idx] = *dir_entry;
        block_idx
    // 再找间接块
    } else {
        // 申请一个间接块
        fs.apply_indirect_data_block(parent_inode);

        // 在间接块中，找是否有空闲目录项
        let indirect_find = self::find_available_entry(parent_inode.get_indirect_data_blocks(), buf, disk);
        ASSERT!(indirect_find.is_some());
        let (block_idx, entry_idx) = indirect_find.unwrap();
        dir_list[entry_idx] = *dir_entry;
        // 直接块的长度  + 间接块的下标
        parent_inode.get_direct_data_blocks_ref().len() + block_idx
    };


    let target_block_lba = &mut parent_inode.get_data_blocks()[block_idx];
    // 找到的数据块LBA是空的
    if target_block_lba.is_empty() {
        // 申请一个数据块
        *target_block_lba = fs.data_block_pool.apply_block(1);
    }

    // 写入 目录项 到硬盘中
    disk.write_sector(buf, *target_block_lba, 1);

    // 增加当前文件的大小
    parent_inode.i_size += size_of::<DirEntry>() as u32;
    // 如果是直接块找到空闲目录项，那么需要同步inode（直接块的地址放在inode的i_sectors字段中）
    fs.sync_inode(parent_inode);

    // 释放缓冲区的空间
    memory::sys_free(buf.as_ptr() as usize);
}
