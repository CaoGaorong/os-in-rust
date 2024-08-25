use os_in_rust_common::racy_cell::RacyCell;

use crate::{filesystem::constant, thread};

use super::{file::OpenedFile, FileDescriptor, FileError};


/**
 * 整个系统打开的文件
 */
static GLOBAL_FILE_TABLE: RacyCell<FileTable> = RacyCell::new(FileTable::empty());


pub struct FileTable {
    table: [Option<OpenedFile>; constant::MAX_OPENED_FILE_IN_SYSTEM],
}

impl FileTable {
    pub const  fn empty() -> Self {
        const REPEAT_FILE:Option<OpenedFile> = Option::None;
        Self {
            table: [REPEAT_FILE; constant::MAX_OPENED_FILE_IN_SYSTEM],
        }
    }

    /**
     * 在文件表中，注册一个文件
     */
    #[inline(never)]
    pub fn register_file(&mut self, file: OpenedFile) -> Option<usize> {
        let idx = self.get_free_index();
        if idx.is_some() {
            self.set_file(idx.unwrap(), file)
        }
        idx
    }

    #[inline(never)]
    fn get_free_index(&self) -> Option<usize> {
        for (idx, ele) in self.table.iter().enumerate() {
            if ele.is_none() {
                return Option::Some(idx);
            }
        }
        return Option::None;
    }

    #[inline(never)]
    fn set_file(&mut self, idx: usize, file: OpenedFile) {
        self.table[idx] = Option::Some(file);
    }
}


/**
 * 在文件表中，注册一个文件。得到文件表的下标
 */
pub fn register_file(file: OpenedFile) -> Option<usize> {
    let file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    file_table.register_file(file)
}

/**
 * 是否某个文件结构
 */
#[inline(never)]
pub fn release_file(idx: usize) {
    let file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    file_table.table[idx] = Option::None;
}

#[inline(never)]
pub fn get_opened_file(idx: usize) -> Option<&'static mut OpenedFile> {
    let file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    file_table.table[idx].as_mut()
}

/**
 * 根据当前任务的文件描述符，得到对应打开的文件
 */
#[inline(never)]
pub fn get_file_by_fd(fd: FileDescriptor) -> Result<&'static mut OpenedFile, FileError> {
    // 先根据文件描述符找
    let cur_task = &thread::current_thread().task_struct;
    let fd_table = &cur_task.fd_table;
    let global_idx = fd_table.get_global_idx(fd);
    if global_idx.is_none() {
        return Result::Err(FileError::FileDescriptorNotFound);
    }

    // 在全局文件结构表里面查找
    let global_idx = global_idx.unwrap();
    let opened_file = self::get_opened_file(global_idx);
    if opened_file.is_none() {
        return Result::Err(FileError::GlobalFileStructureNotFound);
    }
    return Result::Ok(opened_file.unwrap());
}
