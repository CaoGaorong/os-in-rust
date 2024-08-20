use core::fmt::{Debug, Display};

use os_in_rust_common::{cstr_write, cstring_utils, printkln, ASSERT};

use crate::{filesystem, println};

use super::{sys_call_proxy};


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
    pub fn open(&self, path: &str) -> Result<File, filesystem::FileError> {
        // TODO： 这里append未处理
        File::open(path)
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
        Result::Ok(Self::new(sys_call_proxy::open_file(path)?))
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
    pub fn read(&self, buff: &mut [u8]) {
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
}

impl Drop for File {
    #[inline(never)]
    fn drop(&mut self) {
        let res = sys_call_proxy::close_file(&mut self.file);
        if res.is_err() {
            println!("drop file error. {:?}", res.unwrap_err());
        }
    }
}

/**
 * 删除文件
 */
#[inline(never)]
pub fn remove_file(path: &str) -> Result<(), filesystem::FileError> {
    sys_call_proxy::remove_file(path)
}