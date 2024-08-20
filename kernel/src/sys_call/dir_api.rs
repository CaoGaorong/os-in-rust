
use crate::filesystem::{self};

use super::sys_call_proxy;


#[derive(Debug)]
pub struct ReadDir<'a> {
    dir: filesystem::ReadDir<'a>,
}

impl <'a> ReadDir<'a> {
    
    #[inline(never)]
    pub fn new(dir: filesystem::ReadDir<'a>) -> Self {
        Self {
            dir: dir,
        }
    }

    pub fn get_path(&self) -> &str {
        self.dir.get_path()
    }

    #[inline(never)]
    pub fn iter(&'a mut self) -> ReadDirIterator<'a> {
        ReadDirIterator::new(sys_call_proxy::dir_iter(&mut self.dir).unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.dir.is_empty()
    }


    pub fn get_file_size(&self) -> usize {
        self.dir.get_file_size()
    }
}

#[derive(Debug)]
pub struct ReadDirIterator<'a> {
    iter: filesystem::ReadDirIterator<'a>,
}

impl <'a> ReadDirIterator<'a> {
    #[inline(never)]
    pub fn new(iter: filesystem::ReadDirIterator<'a>) -> Self {
        Self {
            iter
        }
    }
}

impl <'a> Drop for ReadDirIterator<'a> {
    
    #[inline(never)]
    fn drop(&mut self) {
        sys_call_proxy::dir_iter_drop(&mut self.iter)
    }
}


impl <'a>Iterator for ReadDirIterator<'a> {
    type Item = &'a filesystem::DirEntry;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        sys_call_proxy::dir_iter_next(&mut self.iter)
    }
}


/**
 * 创建目录
 */
#[inline(never)]
pub fn create_dir(path: &str) -> Result<(), filesystem::DirError> {
    sys_call_proxy::create_dir(path)
}

/**
 * 递归创建目录
 */
#[inline(never)]
pub fn create_dir_all(path: &str) -> Result<(), filesystem::DirError> {
    sys_call_proxy::create_dir_all(path)
}

/**
 * 读取某个目录
 */
#[inline(never)]
pub fn read_dir(path: &str) -> Result<ReadDir, filesystem::DirError> {
    let read = sys_call_proxy::read_dir(path)?;
    Result::Ok(ReadDir::new(read))
}



/**
 * 删除一个目录
 */
#[inline(never)]
pub fn remove_dir(path: &str) -> Result<(),  filesystem::DirError> {
    sys_call_proxy::remove_dir(path)
}