

use crate::{common::open_file_dto::OpenFileDto, filesystem::{self, FileDescriptor}, println};

use super::sys_call_proxy;


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
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    #[inline(never)]
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    #[inline(never)]
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    #[inline(never)]
    pub fn open(&self, path: &str) -> Result<File, filesystem::FileError> {
        let req = OpenFileDto::new(path, self.append);
        Result::Ok(File::new(sys_call_proxy::open_file(&req)?))
    }

}

/**
 * 暴露的外层操作的API，一个打开的文件
 */
#[derive(Debug)]
pub struct File {
    file: filesystem::File,
}

impl File {

    #[inline(never)]
    fn new(file: filesystem::File) -> Self {
        Self {
            file,
        }
    }

    /**
     * 打开一个文件
     */
    #[inline(never)]
    pub fn open(path: &str) -> Result<Self, filesystem::FileError> {
        let req = OpenFileDto::new(path, false);
        Result::Ok(Self::new(sys_call_proxy::open_file(&req)?))
    }

    /**
     * 创建一个文件
     */
    #[inline(never)]
    pub fn create(path:  &str) -> Result<Self, filesystem::FileError> {
        Result::Ok(Self::new(sys_call_proxy::create_file(path)?))
    }


    /**
     * 设置该文件操作的偏移量
     */
    #[inline(never)]
    pub fn seek(&mut self, from: filesystem::SeekFrom) -> Result<(), filesystem::FileError>{
        sys_call_proxy::seek_file(&mut self.file, from)
    }

    /**
     * 从该文件中读取数据
     */
    #[inline(never)]
    pub fn read(&self, buff: &mut [u8]) -> usize {
        sys_call_proxy::read(self.file.get_file_descriptor(), buff)
    }

    /**
     * 把一个缓冲区的数据，写入到当前的文件中
     */
    #[inline(never)]
    pub fn write(&mut self, buff: &[u8]) {
        sys_call_proxy::write(self.file.get_file_descriptor(), buff)
    }

    pub fn get_path(&self) -> &str {
        self.file.get_path()
    }

    #[inline(never)]
    pub fn get_size(&self) -> Result<usize, filesystem::FileError> {
        sys_call_proxy::file_size(&self.file)
    }

    pub fn get_fd(&self) -> FileDescriptor {
        self.file.get_file_descriptor()
    }

    #[inline(never)]
    pub fn close(&mut self) {
        let res = sys_call_proxy::close_file(&mut self.file);
        if res.is_err() {
            println!("drop file error. {:?}", res.unwrap_err());
        }
    }
}

impl Drop for File {
    #[inline(never)]
    fn drop(&mut self) {
        self.close();
    }
}

/**
 * 删除文件
 */
#[inline(never)]
pub fn remove_file(path: &str) -> Result<(), filesystem::FileError> {
    sys_call_proxy::remove_file(path)
}