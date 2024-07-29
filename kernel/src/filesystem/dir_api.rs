use core::mem::size_of;

use os_in_rust_common::{constants, cstr_write, cstring_utils};

use crate::{
    filesystem::{dir_entry, fs}, memory}
;

use super::{
    constant, dir, dir_entry::{DirEntry, DirEntrySearchReq}, inode::{self, OpenedInode}
};

#[derive(Debug)]
pub enum DirError {
    DirPathIllegal,
    NotFound,
    ParentDirNotExists,
    AlreadyExists,
    DirectoryNotEmpty,
}

#[derive(Debug)]
pub struct ReadDir<'a> {
    /**
     * 该目录对应的inode
     */
    inode: &'a mut OpenedInode,

    /**
     * 该目录的全路径
     */
    path: [u8; constant::MAX_FILE_PATH_LEN],

}

impl <'a> ReadDir<'a> {
    
    #[inline(never)]
    pub fn new(inode: &'a mut OpenedInode, dir_path: &str) -> Self {
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
    pub fn iter(&'a mut self) -> ReadDirIterator {
        let buf: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
        ReadDirIterator::new(self.inode, buf)
    }
}

#[derive(Debug)]
pub struct ReadDirIterator<'a> {
    inode: &'a mut OpenedInode,
    block_idx: usize,
    dir_entry_buf: &'a mut [u8; constants::DISK_SECTOR_SIZE],
    dir_entry_idx: usize,
}

impl <'a> ReadDirIterator<'a> {
    #[inline(never)]
    pub fn new(inode: &'a mut OpenedInode, dir_entry_buf: &'a mut [u8; constants::DISK_SECTOR_SIZE]) -> Self {
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
        inode::inode_close(fs::get_filesystem(), self.inode);
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
    let parent_dir_path = &path[..last_slash_idx.max(1)];
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
    let dir_entry = dir_entry::do_search_dir_entry(fs, parent_dir_inode, DirEntrySearchReq::build().entry_name(dir_entry_name));
    // 如果已经存在了，报错
    if dir_entry.is_some() {
        // 把这个parent dir inode关闭掉
        inode::inode_close(fs, parent_dir_inode);
        return Result::Err(DirError::AlreadyExists);
    }

    let dir_inode = dir::mkdir(fs, parent_dir_inode, dir_entry_name);
    // 把这个parent dir inode关闭掉
    inode::inode_close(fs, parent_dir_inode);
    // 关闭当前inode
    inode::inode_close(fs, dir_inode);
    return Result::Ok(());
}

#[inline(never)]
pub fn create_dir_all(path: &str) -> Result<(), DirError> {
    if !path.starts_with("/") {
        return Result::Err(DirError::DirPathIllegal);
    }
    let fs = fs::get_filesystem();
    
    // 根目录为基准目录
    let mut base_inode = inode::inode_open(fs, fs.super_block.root_inode_no);

    // 使用/分隔每个目录项
    let mut dir_entry_split = path.split("/");
    // 遍历每个entry
    while let Option::Some(entry_name) = dir_entry_split.next() {
        if entry_name.is_empty() {
            continue;
        }
        let search_result = dir_entry::do_search_dir_entry(fs, base_inode, DirEntrySearchReq::build().entry_name(entry_name));
        // 如果该目录项已经存在了，搜索下一层
        if search_result.is_some() {
            // 关掉inode
            inode::inode_close(fs, base_inode);
            // 打开找到了inode
            base_inode = inode::inode_open(fs, search_result.unwrap().i_no);
            continue;
        }
        // 创建子目录
        let sub_dir_entry = dir::mkdir(fs, base_inode, entry_name);
        inode::inode_close(fs, base_inode);
        // 再次遍历基于子目录
        base_inode = sub_dir_entry;
    }

    inode::inode_close(fs, base_inode);
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


/**
 * 删除一个目录
 */
#[inline(never)]
pub fn remove_dir(path: &str) -> Result<(),  DirError> {
    let fs = fs::get_filesystem();
    let dir = self::read_dir(path)?;
    let dir_inode = dir.inode;
    // 如果存在数据，无法删除
    if dir_inode.i_size > (size_of::<DirEntry>() * 2).try_into().unwrap() {
        inode::inode_close(fs, dir_inode);
        return Result::Err(DirError::DirectoryNotEmpty);
    }

    // 找到父目录
    let parent_dir = dir_entry::parent_entry(dir_inode);
    let parent_dir_inode = inode::inode_open(fs, parent_dir.i_no);

    // 指定父目录，删除当前目录项
    let succeed = dir_entry::remove_dir_entry(fs, parent_dir_inode, DirEntrySearchReq::build().i_no(dir_inode.i_no));
    inode::inode_close(fs, dir_inode);
    // inode::inode_close(fs, dir_inode);
    if !succeed {
        inode::inode_close(fs, parent_dir_inode);
        return Result::Err(DirError::NotFound);
    }
    // 父目录占用的空间减少
    parent_dir_inode.i_size -= size_of::<DirEntry>() as u32;
    inode::sync_inode(fs, parent_dir_inode);
    // 关闭打开的父目录
    inode::inode_close(fs, parent_dir_inode);

    return Result::Ok(());
}