use core::mem::size_of;

use os_in_rust_common::{constants, cstr_write, cstring_utils, MY_PANIC};

use crate::memory;

use super::{
    constant, dir, dir_entry::{self, DirEntry, DirEntrySearchReq}, file_util, fs, inode::{self, OpenedInode}
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

// impl <'a> Drop for ReadDir<'a> {
//     fn drop(&mut self) {
//         inode::inode_close(fs::get_filesystem(), self.inode);
//     }
// }

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
    pub fn iter(&'a mut self) -> ReadDirIterator<'a> {
        let buf: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
        ReadDirIterator::new(self.inode, buf)
    }

    #[inline(never)]
    pub fn iter_ignore_drop(&'a mut self) -> ReadDirIterator<'a> {
        let mut iter = self.iter();
        iter.ignore_drop = true;
        return iter;
    }

    pub fn is_empty(&self) -> bool {
        // 如果inode的数据区，只有两个目录，那么说明空了
        self.inode.i_size <= (size_of::<DirEntry>() * 2).try_into().unwrap()
    }

    /**
     * 关掉这个inode
     */
    #[inline(never)]
    fn close(&mut self) {
        let fs = fs::get_filesystem();
        inode::inode_close(fs, self.inode);
    }

    pub fn get_file_size(&self) -> usize {
        self.inode.i_size.try_into().unwrap()
    }
}

#[derive(Debug)]
pub struct ReadDirIterator<'a> {
    inode: &'a mut OpenedInode,
    block_idx: usize,
    dir_entry_buf: &'a mut [u8; constants::DISK_SECTOR_SIZE],
    dir_entry_idx: usize,
    ignore_drop: bool,
}

impl <'a> ReadDirIterator<'a> {
    #[inline(never)]
    pub fn new(inode: &'a mut OpenedInode, dir_entry_buf: &'a mut [u8; constants::DISK_SECTOR_SIZE]) -> Self {
        Self {
            inode,
            block_idx: 0,
            dir_entry_buf: dir_entry_buf,
            dir_entry_idx: 0,
            ignore_drop: false,
        }
    }

    #[inline(never)]
    pub fn drop(&mut self) {
        inode::inode_close(fs::get_filesystem(), self.inode);
        memory::sys_free(self.dir_entry_buf.as_ptr() as usize);
    }
}

impl <'a> Drop for ReadDirIterator<'a> {
    
    #[inline(never)]
    fn drop(&mut self) {
        // 不要drop
        if self.ignore_drop {
            return;
        }
        self.drop();
    }
}


impl <'a>Iterator for ReadDirIterator<'a> {
    type Item = &'a DirEntry;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        let data_blocks = self.inode.get_data_blocks_ref();
        // 遍历所有的数据块
        while self.block_idx < data_blocks.len() && !data_blocks[self.block_idx].is_empty() {
            // 如果是数据扇区内的，第一个目录项，那么加载一下
            if self.dir_entry_idx == 0 {
                let disk = unsafe { &mut *fs::get_filesystem().base_part.from_disk };
                disk.read_sectors(data_blocks[self.block_idx], 1, self.dir_entry_buf);
            }
            let dir_entry_list = unsafe { core::slice::from_raw_parts(self.dir_entry_buf as *const _ as *const DirEntry, self.dir_entry_buf.len() / size_of::<DirEntry>()) };
            
            if self.dir_entry_idx >= dir_entry_list.len() {
                MY_PANIC!("idx:{}, len:{}", self.dir_entry_idx, dir_entry_list.len());
            }

            // 我们找到了目录项
            let target_dir_entry = &dir_entry_list[self.dir_entry_idx];

            // 目录项所在的目录项列表下标，往后走
            self.dir_entry_idx += 1;

            // 如果走完了当前扇区的所有目录项
            if self.dir_entry_idx >= dir_entry_list.len() {
                // 往后走
                self.block_idx += 1;
                // 目录项下标清零
                self.dir_entry_idx = 0;
            }

            if !target_dir_entry.is_empty() {
                return Option::Some(target_dir_entry);
            }
        };
        return Option::None;
    }
}

#[inline(never)]
pub fn create_dir(path: &str) -> Result<(), DirError> {
    if !path.starts_with("/") {
        return Result::Err(DirError::DirPathIllegal);
    }
    if path == "/" {
        return Result::Err(DirError::DirPathIllegal);
    }
    
    let fs = fs::get_filesystem();

    let split_res = file_util::split_file_path(path);
    if split_res.is_none() {
        return Result::Err(DirError::DirPathIllegal);
    }

    let (parent_dir_path, dir_entry_name) = split_res.unwrap();

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

    dir::mkdir(fs, parent_dir_inode, dir_entry_name);
    // 把这个parent dir inode关闭掉
    inode::inode_close(fs, parent_dir_inode);
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
        let sub_dir_inode = dir::mkdir(fs, base_inode, entry_name);
        inode::inode_close(fs, base_inode);

        // 再次遍历基于子目录
        base_inode = inode::inode_open(fs, sub_dir_inode);
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
    let mut dir_to_remove = self::read_dir(path)?;
    // 如果存在数据，无法删除
    if !dir_to_remove.is_empty() {
        dir_to_remove.close();
        return Result::Err(DirError::DirectoryNotEmpty);
    }

    // 找到父目录
    let parent_dir_inode = dir_entry::parent_entry(dir_to_remove.inode);

    let parent_dir_inode = inode::inode_open(fs, parent_dir_inode);

    // 指定父目录，删除当前目录项
    let succeed = dir_entry::remove_dir_entry(fs, parent_dir_inode, DirEntrySearchReq::build().i_no(dir_to_remove.inode.i_no));
    // 已经删除完了，关闭那个删除过的inode
    dir_to_remove.close();
    if !succeed {
        inode::inode_close(fs, parent_dir_inode);
        return Result::Err(DirError::NotFound);
    }
    // 父目录操作完成后，保存到硬盘
    inode::sync_inode(fs, parent_dir_inode);
    // 关闭打开的父目录
    inode::inode_close(fs, parent_dir_inode);

    return Result::Ok(());
}