use core::slice;
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    mem::size_of,
};

use kernel::filesystem::{constant, dir::DirEntry, inode::{Inode, OpenedInode}, superblock::SuperBlock};
use os_in_rust_common::{constants, domain::LbaAddr};
pub const DISK_FILE_PATH: &str = "/Users/jackson/MyProjects/rust/os-in-rust/build/hd80M.img";
use lazy_static::lazy_static;

lazy_static!{
    pub static ref INODE_TABLE: [u8; size_of::<Inode>() * constant::MAX_FILE_PER_FS as usize] = read_inode_table();
}
/**
 * 读取硬盘中的超级块
 */
pub fn read_super_block() -> Box<[u8; size_of::<SuperBlock>()]> {
    let mut file = File::open(DISK_FILE_PATH).expect("failed to open disk file");
    file.seek(std::io::SeekFrom::Start(
        (59895 * constants::DISK_SECTOR_SIZE) as u64,
    ))
    .expect("failed to seek");
    let mut reader = BufReader::new(&mut file);
    let mut buf = [0x0u8; size_of::<SuperBlock>()];
    reader.read(&mut buf).expect("failed to read file");
    Box::new(buf)
}

/**
 * 读取inode列表
 */
pub fn read_inode_table() -> [u8; size_of::<Inode>() * constant::MAX_FILE_PER_FS as usize] {
    let super_block = read_super_block();
    let super_block = super_block.as_ref();
    let super_block = unsafe { &*(super_block.as_ptr() as *const SuperBlock) };

    // 读取inode表
    let mut file = File::open(DISK_FILE_PATH).expect("failed to open hd80M.img");
    file.seek(std::io::SeekFrom::Start(
        (usize::from(super_block.inode_table_lba) * constants::DISK_SECTOR_SIZE) as u64,
    ))
    .expect("failed to seek file");
    let mut reader = BufReader::new(&mut file);
    let mut arr = [0x00u8; size_of::<Inode>() * constant::MAX_FILE_PER_FS as usize];
    // 读取文件
    reader.read_exact(&mut arr).expect("failed to read data");

    arr
    // Box::from(arr)
}


/**
 * 读取磁盘的某个LBA地址
 */
pub fn read_disk(disk: &mut File, lba: LbaAddr) -> [u8; constants::DISK_SECTOR_SIZE] {
    disk.seek(std::io::SeekFrom::Start((usize::from(lba) * constants::DISK_SECTOR_SIZE) as u64)).expect("failed to seek file in bytes");
    let mut reader = BufReader::new(disk);
    let mut arr = [0x00u8; constants::DISK_SECTOR_SIZE];
    // 读取出间接块的数据
    reader.read_exact(&mut arr).expect("failed to read data");
    arr
}


/**
 * 加载inode
 */
pub fn load_inode(disk: &mut File,  inode: Inode) -> OpenedInode {
    let mut opened_inode = OpenedInode::new(inode);
    if !inode.indirect_sector.is_empty() {
        // 读取出间接块里面的数据
        let indirect_data = self::read_disk(disk, inode.indirect_sector);
        // 转成多个LBA地址
        let indirect_data_lbas = unsafe { slice::from_raw_parts(indirect_data.as_ptr() as *const LbaAddr, indirect_data.len() / size_of::<LbaAddr>()) };
        // 复制到间接块数据中
        opened_inode.get_indirect_data_blocks().copy_from_slice(indirect_data_lbas)
    }
    opened_inode
}



/**
 * 搜索路径为file_path的文件
 * (bool, String): 是否找到，搜索过的路径
 */
pub fn search_file(file_path: &str) -> (bool, String) {
    if !file_path.starts_with("/") {
        panic!("文件目录必须从根目录开始");
    }
    let inode_table = self::read_inode_table();
    let inode_table = unsafe {
        std::slice::from_raw_parts(
            inode_table.as_ptr() as *const Inode,
            inode_table.len() / size_of::<Inode>(),
        )
    };
    let mut disk_file = File::open(DISK_FILE_PATH).expect("failed to open disk file");

    // 取出根目录inode
    let mut base_inode = load_inode(&mut disk_file, inode_table[0]);

    let mut searched_path = String::new();
    let mut path_split = file_path.split("/");
    while let Option::Some(entry_name) = path_split.next() {
        if entry_name.is_empty() {
            continue;
        }
        let found_entry = self::find_entry_in_data_block(base_inode.get_data_blocks(), entry_name, &mut disk_file);
        if found_entry.is_none() {
            return (false, searched_path);
        }
        let found_entry = found_entry.unwrap();
        // 加载目录项对应的inode
        base_inode = self::load_inode(&mut disk_file, inode_table[usize::from(found_entry.i_no)]);

        searched_path.push_str("/");
        searched_path.push_str(found_entry.get_name());
    }
    return (true, searched_path);
}


/**
 * 从众多数据块LBA地址中（每个LBA地址指向的数据块，都是多个目录项），找到名为entry_name的目录项
 */
fn find_entry_in_data_block(data_blocks: &[LbaAddr], entry_name: &str, disk_file: &mut File) -> Option<DirEntry>{
    for data_block_lba in data_blocks {
        if data_block_lba.is_empty() {
            break;
        }
        // 读取出数据块内容
        let indirect_data = self::read_disk(disk_file, *data_block_lba);
        // 转成目录项
        let dir_entry_list: &[DirEntry] = unsafe { slice::from_raw_parts(indirect_data.as_ptr() as *const DirEntry, indirect_data.len() / size_of::<DirEntry>()) };
        let found_entry = dir_entry_list.iter().find(|entry| entry.get_name().eq(entry_name));
        if found_entry.is_some() {
            return Option::Some(*found_entry.unwrap());
        }
    }
    return Option::None;
    
}
