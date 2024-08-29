use core::fmt::{Debug};

use os_in_rust_common::{cstr_write, cstring_utils, printkln, ASSERT};

use crate::{filesystem::{constant, file, fs}, thread};

use super::{dir_entry::{self, DirEntrySearchReq}, file::{FileError, OpenedFile}, file_descriptor::FileDescriptor, file_util, global_file_table, inode};

#[derive(Clone, Copy)]
pub struct OpenOptions {
    write: bool, 
    append: bool,
    read: bool,
}

impl OpenOptions {
    #[inline(never)]
    pub fn new() -> Self {
        Self {
            write: false,
            append: false,
            read: false,
        }
    }

    #[inline(never)]
    pub fn append(&mut self, append: bool) -> Self {
        self.append = append;
        *self
    }

    #[inline(never)]
    pub fn write(&mut self, write: bool) -> Self {
        self.write = write;
        *self
    }
    #[inline(never)]
    pub fn read(&mut self, read: bool) -> Self {
        self.read = read;
        *self
    }

    #[inline(never)]
    pub fn open(&self, path: &str) -> Result<File, FileError> {
        let fd = file::open_file(path, self.append)?;
        let file = File::new(fd, path, self.write, self.read);
        return Result::Ok(file);
    }

}

/**
 * 文件操作Seek
 */
#[repr(C)]
#[derive(Clone, Copy)]
pub enum SeekFrom {
    /**
     * 从文件的起始开始偏移
     */
    Start(u32),
}

/**
 * 暴露的外层操作的API，一个打开的文件
 */
#[derive(Debug)]
pub struct File {
    /**
     * 文件描述符
     */
    fd: FileDescriptor,
    /**
     * 文件打开的路径
     */
    path: [u8; constant::MAX_FILE_PATH_LEN],
    /**
     * 是否可写
     */
    write: bool,
    read: bool,

    /**
     * 是否忽略关闭
     */
    ignore_drop: bool,
}
// impl Debug for File {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         printkln!("File(fd: {}, path:{}, write:{}, read:{})", self.fd, self.get_path(), self.write, self.read);
//         return Result::Ok(());
//     }
// }

impl File {

    /**
     * 初始化
     */
    #[inline(never)]
    fn new(fd: FileDescriptor, path: &str, write: bool, read: bool) -> Self {
        let mut file = Self {
            fd: fd,
            path: [0; constant::MAX_FILE_PATH_LEN],
            write: write,
            read: read,
            ignore_drop: false,
        };
        // 保存文件路径
        cstr_write!(&mut file.path, "{}", path);
        file
    }

    /**
     * 打开一个文件
     */
    #[inline(never)]
    pub fn open(path: &str) -> Result<Self, FileError> {
        let fd = file::open_file(path, false)?;
        let file = Self::new(fd, path, false, true);
        return Result::Ok(file);
    }

    #[inline(never)]
    pub fn open_ignore_drop(path: &str) -> Result<Self, FileError> {
        let mut file = Self::open(path)?;
        file.ignore_drop = true;
        Result::Ok(file)
    }

    /**
     * 关闭一个文件
     */
    #[inline(never)]
    pub fn close(&self)  -> Result<(), FileError> {
        // 1. 释放当前进程的文件描述符
        let fd_table = &mut thread::current_thread().task_struct.fd_table;
        let task_fd = fd_table.release_fd(self.fd);
        if task_fd.is_none() {
            return Result::Err(FileError::BadDescriptor);
        }
        let task_fd = task_fd.unwrap();
        let global_idx = task_fd.get_global_idx();
        let opend_file = global_file_table::get_opened_file(global_idx);
        if opend_file.is_none() {
            return Result::Err(FileError::BadDescriptor);
        }
        // 3. 关闭这个文件inode
        let opend_file = opend_file.unwrap();
        opend_file.close_file(fs::get_filesystem());

        // 2. 释放全局的文件结构
        global_file_table::release_file(global_idx);
        return Result::Ok(());
    }

    /**
     * 创建一个文件
     */
    #[inline(never)]
    pub fn create(path:  &str) -> Result<Self, FileError> {
        let fd = file::create_file(path)?;
        let file = Self::new(fd, path, true, false);
        Result::Ok(file)
    }

    #[inline(never)]
    pub fn create_ignore_drop(path: &str) -> Result<Self, FileError> {
        let mut file = Self::create(path)?;
        file.ignore_drop = true;
        Result::Ok(file)
    }

    /**
     * 设置该文件操作的偏移量
     */
    #[inline(never)]
    pub fn seek(&mut self, from: SeekFrom) -> Result<(), FileError> {
        let opened_file = global_file_table::get_file_by_fd(self.fd)?;
        
        let off = match from {
            SeekFrom::Start(start) => start,
            _ => 0,
        };

        opened_file.set_file_off(off);
        return Result::Ok(());
    }

    /**
     * 从该文件中读取数据
     */
    #[inline(never)]
    pub fn read(&self, buff: &mut [u8])  -> Result<usize, FileError> {
        if !self.read {
            return Result::Err(FileError::PermissionDenied);
        }
        let opened_file = global_file_table::get_file_by_fd(self.fd)?;
        let fs = fs::get_filesystem();
        Result::Ok(file::read_file(fs, opened_file, buff))
    }

    /**
     * 把一个缓冲区的数据，写入到当前的文件中
     */
    #[inline(never)]
    pub fn write(&mut self, buff: &[u8]) -> Result<usize, FileError> {
        // 没有写入权限
        if !self.write {
            return Result::Err(FileError::PermissionDenied);
        }
        // 根据文件描述符，找到那个文件
        let opened_file = global_file_table::get_file_by_fd(self.fd)?;
        
        let fs = fs::get_filesystem();

        // 写入文件
        Result::Ok(file::write_file(fs, opened_file, buff))
    }

    pub fn get_path(&self) -> &str {
        let res = cstring_utils::read_from_bytes(&self.path);
        ASSERT!(res.is_some());
        res.unwrap()
    }

    #[inline(never)]
    pub fn get_size(&self) -> Result<usize, FileError> {
        let opened_file =  global_file_table::get_file_by_fd(self.fd)?;
        let opened_inode = opened_file.get_inode();
        Result::Ok(opened_inode.i_size.try_into().unwrap())
    }

    pub fn get_file_descriptor(&self) -> FileDescriptor {
        self.fd
    }
}

impl Drop for File {
    #[inline(never)]
    fn drop(&mut self) {
        // 忽略drop
        if self.ignore_drop {
            return;
        }
        // 文件离开作用域，自动关闭文件
        let res = self.close();
        ASSERT!(res.is_ok());
    }
}

/**
 * 删除文件
 */
#[inline(never)]
pub fn remove_file(path: &str) -> Result<(), FileError> {
    if path == "/" {
        return Result::Err(FileError::FilePathIllegal);
    }
    let split_res = file_util::split_file_path(path);
    if split_res.is_none() {
        return Result::Err(FileError::FilePathIllegal);
    }
    let fs: &mut fs::FileSystem = fs::get_filesystem();
    let (dir_path, file_name) = split_res.unwrap();

    // 该文件所在的父目录
    let parent_dir = dir_entry::search_dir_entry(fs, dir_path);
    if parent_dir.is_none() {
        return Result::Err(FileError::ParentDirNotExists);
    }
    let (_, parent_dir_inode) = parent_dir.unwrap();
    
    // 在该父目录下，搜索该文件
    let cur_file_entry = dir_entry::do_search_dir_entry(fs, parent_dir_inode, DirEntrySearchReq::build().entry_name(file_name));
    if cur_file_entry.is_none() {
        inode::inode_close(fs, parent_dir_inode);
        return Result::Err(FileError::NotFound);
    }
    // 该文件的inode
    let cur_file_entry = cur_file_entry.unwrap();
    let cur_file_inode = inode::inode_open(fs, cur_file_entry.i_no);
    
    // 把这个文件的数据扇区LBA地址都加载出来（间接扇区）
    inode::load_indirect_data_block(fs, cur_file_inode);
    // 指定父目录，删除这个文件inode
    file::remove_file(fs, parent_dir_inode, cur_file_inode)?;
    
    // 关闭inode
    inode::inode_close(fs, parent_dir_inode);
    inode::inode_close(fs, cur_file_inode);
    // remove_res
    return Result::Ok(());
}