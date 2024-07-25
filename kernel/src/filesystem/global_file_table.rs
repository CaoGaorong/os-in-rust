use os_in_rust_common::racy_cell::RacyCell;

use crate::filesystem::constant;

use super::file::OpenedFile;


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

pub fn get_opened_file(idx: usize) -> Option<&'static mut OpenedFile> {
    let file_table = unsafe { GLOBAL_FILE_TABLE.get_mut() };
    file_table.table[idx].as_mut()
}

