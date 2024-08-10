use core::fmt::Display;

use os_in_rust_common::{constants, printkln};

/**
 * 标准文件描述符
 */
pub enum StdFileDescriptor {
    /**
     * 标准输入
     */
    StdInputNo = 0x0,
    /**
     * 标准输出
     */
    StdOutputNo = 0x1,
    /**
     * 标准错误
     */
    StdErrorNo = 0x2,
}


#[derive(Debug)]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FileDescriptor {
    value: usize
}

impl Display for FileDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        printkln!("{}", self.value);
        Result::Ok(())
    }
}

impl FileDescriptor {
    pub fn new(value: usize) -> Self {
        Self {
            value,
        }
    }
}

/**
 * 文件描述符表
 */
#[repr(C)]
pub struct FileDescriptorTable {
    data: [Option<usize>; constants::MAX_FILES_PER_PROC],
    start_idx: usize,
}
impl FileDescriptorTable {
    /**
     * 创建一个文件描述符表
     */
    pub fn new() -> Self {
        let mut fd_table = [Option::None; constants::MAX_FILES_PER_PROC];
        fd_table[StdFileDescriptor::StdInputNo as usize] = Option::Some(StdFileDescriptor::StdInputNo as usize);
        fd_table[StdFileDescriptor::StdOutputNo as usize] = Option::Some(StdFileDescriptor::StdOutputNo as usize);
        fd_table[StdFileDescriptor::StdErrorNo as usize] = Option::Some(StdFileDescriptor::StdErrorNo as usize);
        Self {
            data: fd_table,
            start_idx: fd_table.iter().map(|d| d.is_some()).count(),
        }
    }

    /**
     * 得到文件描述符（不包括标准输入、输出等内建的）
     */
    pub fn get_file_descriptors(&self) -> &[Option<usize>] {
        &self.data[self.start_idx ..]
    }

    /**
     * 在文件描述符表中，找到空位
     */
    pub fn get_free_slot(&self) -> Option<usize> {
        for (idx, fd) in self.data.iter().enumerate() {
            // 找到了空位，返回下标
            if fd.is_none() {
                return Option::Some(idx);
            }
        }
        return Option::None;
    }

    /**
     * 给当前进程的文件描述符表，安装一个全局文件描述符
     */
    pub fn install_fd(&mut self, global_file_descriptor: usize) -> Option<FileDescriptor> {
        // 当前的文件描述符表，找到空位
        let slot_idx = self.get_free_slot();
        if slot_idx.is_none() {
            return Option::None;
        }
        let slot_idx = slot_idx.unwrap();
        // 填充
        self.data[slot_idx] = Option::Some(global_file_descriptor);
        
        // 数组下标，就是文件描述符
        Option::Some(FileDescriptor::new(slot_idx))
    }

    pub fn get_global_idx(&self, fd: FileDescriptor) -> Option<usize> {
        self.data[fd.value]
    }

    /**
     * 释放某个文件描述符。得到全局的文件结构表下标
     */
    pub fn release_fd(&mut self, fd: FileDescriptor) -> Option<usize> {
        let global_idx = self.data[fd.value];
        // 清除
        self.data[fd.value] = Option::None;
        global_idx
    }

}

impl Display for FileDescriptorTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        printkln!("{:?}", self.data);
        Result::Ok(())
    }
}