use os_in_rust_common::{constants, ASSERT};

use super::file::StdFileDescriptor;


/**
 * 文件描述符表
 */
#[repr(transparent)]
pub struct FileDescriptorTable {
    data: [Option<u32>; constants::MAX_FILES_PER_PROC],
}
impl FileDescriptorTable {
    /**
     * 创建一个文件描述符表
     */
    pub fn new() -> Self {
        let mut fd_table = [Option::None; constants::MAX_FILES_PER_PROC];
        fd_table[StdFileDescriptor::StdInputNo as usize] = Option::Some(StdFileDescriptor::StdInputNo as u32);
        fd_table[StdFileDescriptor::StdOutputNo as usize] = Option::Some(StdFileDescriptor::StdOutputNo as u32);
        fd_table[StdFileDescriptor::StdErrorNo as usize] = Option::Some(StdFileDescriptor::StdErrorNo as u32);
        Self {
            data: fd_table,
        }
    }

    /**
     * 在文件描述符表中，找到空位
     */
    pub fn get_free_slot(&self) -> Option<usize> {
        for (idx, fd) in self.data.iter().enumerate() {
            // 找到了空位，返回下标
            if fd.is_some() {
                return Option::Some(idx);
            }
        }
        return Option::None;
    }

    /**
     * 给当前进程的文件描述符表，安装一个全局文件描述符
     */
    pub fn install_fd(&mut self, global_file_descriptor: u32) {
        // 当前的文件描述符表，找到空位
        let slot_idx = self.get_free_slot();
        ASSERT!(slot_idx.is_some());
        let slot_idx = slot_idx.unwrap();
        // 填充
        self.data[slot_idx] = Option::Some(global_file_descriptor);
    }


}