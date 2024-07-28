use os_in_rust_common::{cstr_write, printkln, ASSERT};

use crate::{filesystem::{constant, file, fs}, thread};

use super::{dir_entry::{self, DirEntrySearchReq}, file::{FileError, OpenedFile}, file_descriptor::FileDescriptor, global_file_table};

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
}

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

    /**
     * 关闭一个文件
     */
    #[inline(never)]
    fn close(&self)  -> Result<(), FileError> {
        // 1. 释放当前进程的文件描述符
        let fd_table = &mut thread::current_thread().task_struct.fd_table;
        let global_idx = fd_table.release_fd(self.fd);
        if global_idx.is_none() {
            return Result::Err(FileError::BadDescriptor);
        }
        let global_idx = global_idx.unwrap();
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


    /**
     * 设置该文件操作的偏移量
     */
    #[inline(never)]
    pub fn seek(&mut self, from: SeekFrom) -> Result<(), FileError>{
        let opened_file = self.get_opened_file()?;
        
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
        let opened_file = self.get_opened_file()?;
        let fs = fs::get_filesystem();
        file::read_file(fs, opened_file, buff)
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
        let opened_file = self.get_opened_file()?;
        
        let fs = fs::get_filesystem();

        // 写入文件
        file::write_file(fs, opened_file, buff)
    }

    /**
     * 根据当前任务的文件描述符，得到对应打开的文件
     */
    #[inline(never)]
    fn get_opened_file(&self) -> Result<&'static mut OpenedFile, FileError> {
        // 先根据文件描述符找
        let fd_table = &thread::current_thread().task_struct.fd_table;
        let global_idx = fd_table.get_global_idx(self.fd);
        if global_idx.is_none() {
            return Result::Err(FileError::FileDescriptorNotFound);
        }

        // 在全局文件结构表里面查找
        let global_idx = global_idx.unwrap();
        let opened_file = global_file_table::get_opened_file(global_idx);
        if opened_file.is_none() {
            return Result::Err(FileError::GlobalFileStructureNotFound);
        }
        return Result::Ok(opened_file.unwrap());
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // printkln!("drop fd: {:?}", self.fd);
        // 文件离开作用域，自动关闭文件
        let res = self.close();
        ASSERT!(res.is_ok());
    }
}

pub fn remove_file(path: &str) -> Result<(), FileError> {
    if path.ends_with("/") {
        return Result::Err(FileError::IsADirectory);
    }
    // 最后一个斜线的下标
    let last_slash_idx = path.rfind("/");
    if last_slash_idx.is_none() {
        return Result::Err(FileError::FilePathIllegal);
    }
    let last_slash_idx = last_slash_idx.unwrap();
    // 父目录的路径
    let parent_dir_path = &path[..last_slash_idx.max(1)];
    // 要创建的目录项的名称
    let file_entry_name = &path[last_slash_idx + 1..];

    let fs = fs::get_filesystem();
    // 先找父目录项
    let parent_dir_entry = dir_entry::search_dir_entry(fs, parent_dir_path);
    if parent_dir_entry.is_none() {
        return Result::Err(FileError::ParentDirNotExists);
    }

    // 找该文件的目录项
    let (_, parent_dir_inode) = parent_dir_entry.unwrap();
    let file_entry = dir_entry::do_search_dir_entry(fs, parent_dir_inode, DirEntrySearchReq::build().entry_name(file_entry_name));
    if file_entry.is_none() {
        return Result::Err(FileError::NotFound);
    }

    // 删除这个文件


    return Result::Ok(());
}