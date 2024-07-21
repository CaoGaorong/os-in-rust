use os_in_rust_common::{cstr_write, ASSERT};

use crate::{filesystem::{constant, file, fs}, thread};

use super::{file::{FileError, OpenedFile}, file_descriptor::{self, FileDescriptor}, global_file_table};

#[derive(Clone, Copy)]
pub struct OpenOptions {
    write: bool, 
    append: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            write: false,
            append: false,
        }
    }

    pub fn append(&mut self, append: bool) -> Self {
        self.append = append;
        *self
    }

    pub fn write(&mut self, write: bool) -> Self {
        self.write = write;
        *self
    }

    pub fn open(&self, path: &str) -> Result<File, FileError> {
        let fd = file::open_file(path, self.append)?;
        let file = File::new(fd, path, self.write);
        return Result::Ok(file);
    }

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
}

impl File {

    /**
     * 初始化
     */
    #[inline(never)]
    fn new(fd: FileDescriptor, path: &str, write: bool) -> Self {
        let mut file = Self {
            fd: fd,
            path: [0; constant::MAX_FILE_PATH_LEN],
            write: write,
        };
        // 保存文件路径
        cstr_write!(&mut file.path, "{}", path);
        file
    }

    /**
     * 打开一个文件
     */
    pub fn open(path: &str) -> Result<Self, FileError> {
        let fd = file::open_file(path, false)?;
        let file = Self::new(fd, path, false);
        return Result::Ok(file);
    }

    /**
     * 创建一个文件
     */
    #[inline(never)]
    pub fn create(path:  &str) -> Result<Self, FileError> {
        let fd = file::create_file(path)?;
        let file = Self::new(fd, path, true);
        Result::Ok(file)
    }


    /**
     * 设置该文件操作的偏移量
     */
    pub fn seek(&mut self, from: u32) -> Result<(), FileError>{
        let opened_file = self.get_opened_file()?;
        opened_file.set_file_off(from);
        return Result::Ok(());
    }

    /**
     * 从该文件中读取数据
     */
    pub fn read(&self, buff: &mut [u8])  -> Result<usize, FileError> {
        let opened_file = self.get_opened_file()?;
        let fs = fs::get_filesystem();
        ASSERT!(fs.is_some());
        let fs = fs.unwrap();
        file::read_file(fs, opened_file, buff, buff.len());
        return Result::Ok(1);
    }

    /**
     * 把一个缓冲区的数据，写入到当前的文件中
     */
    pub fn write(&mut self, buff: &[u8]) -> Result<usize, FileError> {
        // 没有写入权限
        if self.write == false {
            return Result::Err(FileError::PermissionDenied);
        }
        let opened_file = self.get_opened_file()?;
        
        let fs = fs::get_filesystem();
        ASSERT!(fs.is_some());
        let fs = fs.unwrap();

        // 写入文件
        file::write_file(fs, opened_file, buff)
    }

    /**
     * 获取当前打开的文件
     */
    fn get_opened_file(&self) -> Result<&'static mut OpenedFile, FileError> {
        // 找不到文件结构
        let fd_table = &thread::current_thread().task_struct.fd_table;
        let global_idx = fd_table.get_global_idx(self.fd);
        if global_idx.is_none() {
            return Result::Err(FileError::Uncategorized);
        }

        let global_idx = global_idx.unwrap();
        let opened_file = global_file_table::get_opened_file(global_idx);
        if opened_file.is_none() {
            return Result::Err(FileError::Uncategorized);
        }
        return Result::Ok(opened_file.unwrap());
    }
}