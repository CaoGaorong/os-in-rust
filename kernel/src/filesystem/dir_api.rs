use core::mem::size_of;

use os_in_rust_common::{constants, cstr_write, cstring_utils, ASSERT};

use crate::{
    filesystem::{dir_entry, fs}, memory}
;

use super::{
    constant, dir, dir_entry::DirEntry, inode::OpenedInode
};

#[derive(Debug)]
pub enum DirError {
    DirPathIllegal,
    NotFound,
    ParentDirNotExists,
    AlreadyExists,
}

#[derive(Debug)]
pub struct ReadDir {
    /**
     * 该目录对应的inode
     */
    inode: &'static OpenedInode,

    /**
     * 该目录的全路径
     */
    path: [u8; constant::MAX_FILE_PATH_LEN],

}

impl ReadDir {
    
    #[inline(never)]
    pub fn new(inode: &'static OpenedInode, dir_path: &str) -> Self {
        let mut dir = Self {
            inode: inode,
            path: [0; constant::MAX_FILE_PATH_LEN],
        };
        // 保存文件路径
        cstr_write!(&mut dir.path, "{}", dir_path);
        dir
    }

    pub fn get_path(&self) -> &str {
        return cstring_utils::read_from_bytes(&self.path).unwrap();
    }

    #[inline(never)]
    pub fn iter(&self) -> ReadDirIterator {
        let buf: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
        ReadDirIterator::new(self.inode, buf)
    }
}

#[derive(Debug)]
pub struct ReadDirIterator<'a> {
    inode: &'static OpenedInode,
    block_idx: usize,
    dir_entry_buf: &'a mut [u8; constants::DISK_SECTOR_SIZE],
    dir_entry_idx: usize,
}

impl <'a> ReadDirIterator<'a> {
    #[inline(never)]
    pub fn new(inode: &'static OpenedInode, dir_entry_buf: &'static mut [u8; constants::DISK_SECTOR_SIZE]) -> Self {
        Self {
            inode,
            block_idx: 0,
            dir_entry_buf: dir_entry_buf,
            dir_entry_idx: 0,
        }
    }
}

impl <'a> Drop for ReadDirIterator<'a> {
    
    #[inline(never)]
    fn drop(&mut self) {
        memory::sys_free(self.dir_entry_buf.as_ptr() as usize);
    }
}


impl <'a>Iterator for ReadDirIterator<'a> {
    type Item = DirEntry;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        let data_blocks = self.inode.get_data_blocks_ref();
        // 如果遍历到这一块，没有数据了，那么就是遍历完了
        if self.block_idx >= data_blocks.len() || data_blocks[self.block_idx].is_empty() {
            return Option::None;
        }

        // 如果是数据扇区内的，第一个目录项，那么加载一下
        if self.dir_entry_idx == 0 {
            let disk = unsafe { &mut *fs::get_filesystem().base_part.from_disk };
            disk.read_sectors(data_blocks[self.block_idx], 1, self.dir_entry_buf);
        }
        let dir_entry_list = unsafe { core::slice::from_raw_parts(self.dir_entry_buf as *const _ as *const DirEntry, self.dir_entry_buf.len() / size_of::<DirEntry>()) };
        // 我们找到了目录项
        let target_dir_entry = dir_entry_list[self.dir_entry_idx];
        // 如果目标目录项是空的，说明已经遍历结束了
        if target_dir_entry.is_empty() {
            return Option::None;
        }

        // 目录项所在的目录项列表下标，往后走
        self.dir_entry_idx += 1;

        // 如果走完了当前扇区的所有目录项
        if self.dir_entry_idx >= dir_entry_list.len() {
            // 往后走
            self.block_idx += 1;
            // 目录项下标清零
            self.dir_entry_idx = 0;
        }

        return Option::Some(target_dir_entry);
    }
}

#[inline(never)]
pub fn create_dir(path: &str) -> Result<(), DirError> {
    if !path.starts_with("/") {
        return Result::Err(DirError::DirPathIllegal);
    }
    let fs = fs::get_filesystem();

    let mut path = path;
    // 去掉最后一个斜线
    if path.ends_with("/") {
        path = &path[..path.len() - 1];
    }
    // 最后一个斜线的下标
    let last_slash_idx = path.rfind("/");
    if last_slash_idx.is_none() {
        return Result::Err(DirError::DirPathIllegal);
    }
    let last_slash_idx = last_slash_idx.unwrap();
    // 父目录的路径
    let parent_dir_path = &path[..last_slash_idx];
    // 要创建的目录项的名称
    let dir_entry_name = &path[last_slash_idx + 1..];

    // 父目录的inode
    let parent_dir = dir_entry::search_dir_entry(fs, parent_dir_path);
    // 父目录不存在，报错
    if parent_dir.is_none() {
        return Result::Err(DirError::ParentDirNotExists);
    }
    let (_, parent_dir_inode) = parent_dir.unwrap();
    // 搜索要创建的inode
    let dir_entry = dir_entry::do_search_dir_entry(fs, parent_dir_inode, dir_entry_name);
    // 如果已经存在了，报错
    if dir_entry.is_some() {
        return Result::Err(DirError::AlreadyExists);
    }

    dir::mkdir(fs, parent_dir_inode, dir_entry_name);
    return Result::Ok(());
}

#[inline(never)]
pub fn create_dir_all(path: &str) -> Result<(), DirError> {
    if !path.starts_with("/") {
        return Result::Err(DirError::DirPathIllegal);
    }
    let fs = fs::get_filesystem();

    let root_dir = fs.get_root_dir();
    let mut base_inode = root_dir.get_inode_ref();

    // 使用/分隔每个目录项
    let mut dir_entry_split = path.split("/");
    // 遍历每个entry
    while let Option::Some(entry_name) = dir_entry_split.next() {
        if entry_name.is_empty() {
            continue;
        }
        // 创建子目录
        let sub_dir_entry = dir::mkdir(fs, base_inode, entry_name);
        // 基于子目录
        base_inode = fs.inode_open(sub_dir_entry.i_no);
    }
    return Result::Ok(());
}

/**
 * 读取某个目录
 */
#[inline(never)]
pub fn read_dir(path: &str) -> Result<ReadDir, DirError> {
    // 根据名称，搜索到这个目录项对应的inode
    let entry_inode = self::search_dir_entry(path)?;
    let dir = ReadDir::new(entry_inode, path);
    Result::Ok(dir)
}

#[inline(never)]
fn search_dir_entry(path: &str) -> Result<&'static mut OpenedInode, DirError> {
    if !path.starts_with("/") {
        return Result::Err(DirError::DirPathIllegal);
    }
    let fs = fs::get_filesystem();

    // 搜索到这个文件
    let searched_file = dir_entry::search_dir_entry(fs, path);
    if searched_file.is_none() {
        return Result::Err(DirError::NotFound);
    }
    return Result::Ok(searched_file.unwrap().1);
}
