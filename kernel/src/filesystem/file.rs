
use core::{fmt::Display, mem::size_of};

use os_in_rust_common::{constants, printkln, ASSERT};


use crate::{console_println, memory, thread};
use super::{
    constant, dir_entry::{self, DirEntrySearchReq}, file_descriptor::FileDescriptor, file_util, fs::{self, FileSystem}, global_file_table, inode::{self, OpenedInode}, DirEntry, FileType
};

/**
 * 文件系统的，文件的结构
 */
pub struct OpenedFile {
    /**
     * 这个打开的文件，底层指向的inode节点
     */
    inode: &'static mut OpenedInode,
    /**
     * 操作的文件的偏移量（单位字节）
     */
    file_off: u32,
}

impl OpenedFile {
    pub fn new(inode: &'static mut OpenedInode, append: bool) -> Self {
        let file_size = inode.i_size;
        Self {
            inode,
            file_off: if append {file_size} else {0},
        }
    }

    #[inline(never)]
    pub fn set_file_off(&mut self, off: u32) {
        self.file_off = off;
    }

    /**
     * 该文件再次打开
     */
    pub fn reopen(&mut self) {
        self.inode.reopen();
    }
    
    /**
     * 关闭某个文件
     */
    pub fn close_file(&mut self, fs: &mut FileSystem) {
        inode::inode_close(fs, self.inode);
    }

    pub fn get_inode(&self) -> &OpenedInode {
        self.inode
    }

}


#[derive(Debug)]
pub enum FileError {
    // 文件路径非法
    FilePathIllegal,
    // 文件已存在
    AlreadyExists,
    // 父目录不存在
    ParentDirNotExists,
    // 文件不存在
    NotFound,
    // 无法操作（权限不够）
    Uncategorized,
    // 是一个目录，无法操作
    IsADirectory,
    // 文件数量超过当前任务的限制
    FileExceedTask,
    // 文件数量超过当前系统的限制
    FileExceedSystem,
    // 没有操作权限
    PermissionDenied,
    // 文件描述符找不到
    FileDescriptorNotFound,
    // 全局的文件结构找不到
    GlobalFileStructureNotFound,
    // 文件
    BadDescriptor,

    // 无法删除一个打开中的文件
    CouldNotRemoveAnOpenedFile
}

// pub fn close_file()


#[inline(never)]
pub fn open_file(file_path: &str, append: bool) -> Result<FileDescriptor, FileError>{
    if !file_path.starts_with("/") {
        return Result::Err(FileError::FilePathIllegal);
    }
    let fs = fs::get_filesystem();


    // 搜索到这个文件
    let searched_file = dir_entry::search_dir_entry(fs, file_path);
    if searched_file.is_none() {
        return Result::Err(FileError::NotFound);
    }
    let (_, file_inode) = searched_file.unwrap();

    // // 如果打开的是一个文件夹，不允许打开
    // if searched_file.file_type as FileType == FileType::Directory {
    //     return Result::Err(FileError::IsADirectory);
    // }
    
    // 得到一个打开文件
    let opened_file = OpenedFile::new(file_inode, append);

    // 把这个文件注册到 「系统文件结构数组中」
    let global_file_idx = global_file_table::register_file(opened_file);
    if global_file_idx.is_none() {
        return Result::Err(FileError::FileExceedSystem);
    }

    // 然后安装到当前任务的「文件结构数组」中
    let file_table_idx = global_file_idx.unwrap();
    let fd = thread::current_thread().task_struct.fd_table.install_fd(file_table_idx);
    // 当前任务没有空位了
    if fd.is_none() {
        return Result::Err(FileError::FileExceedTask);
    }
    return Result::Ok(fd.unwrap());
}


/**
 * 指定路径，创建一个文件
 */
#[inline(never)]
pub fn create_file(file_path: &str) -> Result<FileDescriptor, FileError> {
    let fs = fs::get_filesystem();
    // 斜杠结尾的，是目录，不是文件
    if file_path.ends_with("/") {
        return Result::Err(FileError::IsADirectory);
    }

    // 把文件路径，分为父目录路径和文件名称
    let split_result = file_util::split_file_path(file_path);
    if split_result.is_none() {
        return Result::Err(FileError::FilePathIllegal);
    }
    let (dir_path, file_name) = split_result.unwrap();

    // 先搜索一下，目录是否存在
    let search_dir = dir_entry::search_dir_entry(fs, dir_path);
    if search_dir.is_none() {
        return Result::Err(FileError::ParentDirNotExists);
    }
    // 在搜索一下这个文件是否存在
    let dir_inode = search_dir.unwrap().1;
    let search_entry = dir_entry::do_search_dir_entry(fs, dir_inode, DirEntrySearchReq::build().entry_name(file_name));
    if search_entry.is_some() {
        inode::inode_close(fs, dir_inode);
        return Result::Err(FileError::AlreadyExists);
    }

    let opened_inode = dir_entry::create_dir_entry(fs, dir_inode, file_name, FileType::Regular);
    inode::inode_close(fs, dir_inode);
    
    // 得到一个打开文件
    let opened_inode = inode::inode_open(fs, opened_inode);
    let opened_file = OpenedFile::new(opened_inode, false);

    // 把这个文件注册到 「系统文件结构数组中」
    let global_file_idx = global_file_table::register_file(opened_file);
    if global_file_idx.is_none() {
        return Result::Err(FileError::FileExceedSystem);
    }

    // 然后安装到当前任务的「文件结构数组」中
    let file_table_idx = global_file_idx.unwrap();
    let fd = thread::current_thread().task_struct.fd_table.install_fd(file_table_idx);
    // 当前任务没有空位了
    if fd.is_none() {
        return Result::Err(FileError::FileExceedTask);
    }

    return Result::Ok(fd.unwrap());
}

/**
* 把buff数组写入到file文件中，写入bytes个字节
*
           first_sector
       LBA x             x+1            x+2           x+3
           +----------------------------------------------------------+
   disk    |              |              |             |              |
           |              |              |             |              |
           +----------------------------------------------------------+
           ^      +----------------------------------------------+
           |  buf |                                              |
           |      +----------------------------------------------+
           |      ^       ^
           |      |       |
   扇区开始的字节   |     对于buf数组，写入后面扇区的开始数据
                  |
           file.file_off
           要写入硬盘的起始数据
*/
#[inline(never)]
pub fn write_file(fs: &mut FileSystem, file: &mut OpenedFile, buff: &[u8]) -> Result<usize, FileError>{

    let disk = unsafe { &mut *fs.base_part.from_disk };

    let start_data_block_idx = file.file_off as usize / constants::DISK_SECTOR_SIZE;
    // 要写入到文件的最后一个字节，所在该inode数据扇区的下标
    let end_data_block_idx = (file.file_off as usize + buff.len()) / constants::DISK_SECTOR_SIZE;
    // 如果涉及到间接块，需要申请一个间接块
    if end_data_block_idx >= file.inode.get_direct_data_blocks_ref().len() {
        inode::apply_indirect_data_block(fs, file.inode);
    }

    // 要操作的文件偏移量，超过1个扇区的字节数
    let start_bytes_over_sector = file.file_off as usize % constants::DISK_SECTOR_SIZE;
    // 要操作的文件偏移量，在一个扇区中剩余的字节数
    let start_bytes_left_sector = constants::DISK_SECTOR_SIZE - start_bytes_over_sector;

    // 要写入的最后一个字节，超过整扇区的部分（字节数）
    let end_bytes_over_sector = (file.file_off as usize + buff.len()) % constants::DISK_SECTOR_SIZE;

    // 申请单个扇区大小的缓冲区，用于循环读取扇区的数据
    let single_sector_buffer: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
    let mut succeed_bytes = 0usize;

    // 遍历所有的数据块扇区
    for block_idx in start_data_block_idx..=end_data_block_idx {
        // 相对的块下标。从file.file_off所在的块开始，下标为0
        let relative_block_idx = block_idx - start_data_block_idx;
        // 把缓冲区清空
        unsafe { single_sector_buffer.as_mut_ptr().write_bytes(0, single_sector_buffer.len()) };

        // 本次循环写入的字节数量
        let mut bytes_written = constants::DISK_SECTOR_SIZE;
        // 要写入的数据扇区的LBA地址
        let data_block_lba = &mut file.inode.get_data_blocks()[block_idx];
        let mut new_data_block = false;
        // 如果这个数据扇区没有填充过，那么需要申请一个数据块
        if data_block_lba.is_empty() {
            *data_block_lba = fs.data_block_pool.apply_block(1);
            new_data_block = true;
        }

        // 如果是第一个扇区，并且开始写入的字节开始偏移量不是整扇区
        if relative_block_idx == 0 && start_bytes_over_sector > 0 {
            if !new_data_block {
                // 读取出这个扇区
                disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            }
            // 写入的字节数量 = 当前扇区剩余的数量和缓冲区长度的最小值
            bytes_written = start_bytes_left_sector.min(buff.len());
            // 把缓冲区single_sector_buffer的后半部分，使用要写入的数据buff覆盖
            single_sector_buffer[start_bytes_over_sector..start_bytes_over_sector + bytes_written].copy_from_slice(&buff[..bytes_written]);

        // 如果是操作最后一个扇区，并且写入的字节结束偏移量不是整扇区
        } else if block_idx == end_data_block_idx && end_bytes_over_sector > 0 {
            if !new_data_block {
                // 读取出这个扇区
                disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            }
            bytes_written = end_bytes_over_sector;
            // 如果这是最后一个扇区，同时也是第一个扇区
            let sector_start_idx = if block_idx == start_data_block_idx { start_bytes_over_sector } else {0};
            // 缓冲区要操作开始的字节下标
            let buf_start_idx = (constants::DISK_SECTOR_SIZE * relative_block_idx).overflowing_sub(start_bytes_over_sector);
            // 如果溢出了，那开始字节偏移就是0
            let buf_start_idx = if buf_start_idx.1 { 0 } else {buf_start_idx.0};
            
            single_sector_buffer[sector_start_idx..end_bytes_over_sector].copy_from_slice(&buff[buf_start_idx..buf_start_idx + end_bytes_over_sector]);

        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            // 缓冲区开始的字节偏移。
            let buf_start_byte_idx = start_bytes_left_sector as isize + constants::DISK_SECTOR_SIZE as isize * (relative_block_idx as isize - 1);
            ASSERT!(buf_start_byte_idx >= 0);
            let buf_start_byte_idx = buf_start_byte_idx as usize;
            // 缓冲区结束的字节偏移
            let buf_end_byte_idx = start_bytes_left_sector + constants::DISK_SECTOR_SIZE * relative_block_idx;
            single_sector_buffer.copy_from_slice(&buff[buf_start_byte_idx .. buf_end_byte_idx]);
            bytes_written = single_sector_buffer.len();
        }
        disk.write_sector(single_sector_buffer, *data_block_lba, 1);
        succeed_bytes += bytes_written;
    }
    // 释放缓冲区
    memory::sys_free(single_sector_buffer.as_ptr() as usize);
    // 该文件操作的偏移量增加
    file.file_off += succeed_bytes as u32;

    // 当前文件的数据大小发生变化
    file.inode.i_size = file.inode.i_size.max(file.file_off);
    // 把inode元数据同步到硬盘（inode数组）
    inode::sync_inode(fs, file.inode);

    return Result::Ok(succeed_bytes);
}

/**
* 把buff数组写入到file文件中，写入bytes个字节
*
           first_sector
       LBA  x           x + 1          x + 2         x + 3
           +----------------------------------------------------------+
   disk    |              |              |             |              |
           |              |              |             |              |
           +----------------------------------------------------------+
           ^      +----------------------------------------------+
           |  buf |                                              |
           |      +----------------------------------------------+
           |      ^       ^
           |      |       |
   扇区开始的字节   |     对于buf数组，写入后面扇区的开始数据
                  |
        file.file_off % constants::DISK_SECTOR_SIZE
           要写入硬盘的起始数据
*/
#[inline(never)]
pub fn read_file(fs: &mut FileSystem, file: &mut OpenedFile, buff: &mut [u8]) -> Result<usize, FileError>{

    // // 最多读取到文件的末尾
    // let end_byte_off_file = (file.file_off as usize + buff.len()).min(file.inode.i_size as usize);

    let disk = unsafe { &mut *fs.base_part.from_disk };

    let start_data_block_idx = file.file_off as usize / constants::DISK_SECTOR_SIZE;
    // 要写入到文件的最后一个字节，所在该inode数据扇区的下标
    let end_data_block_idx = (file.file_off as usize + buff.len()) / constants::DISK_SECTOR_SIZE;
    // 如果涉及到间接块，那么需要加载间接块的数据
    if end_data_block_idx >= file.inode.get_direct_data_blocks_ref().len() {
        inode::load_indirect_data_block(fs, file.inode);
    }

    // 要操作的文件开始的字节，距离所在扇区开头的偏移量
    let start_bytes_over_sector = file.file_off as usize % constants::DISK_SECTOR_SIZE;
    // 要操作的文件开始的字节，距离所在扇区结束的偏移量
    let start_bytes_away_sector = constants::DISK_SECTOR_SIZE - start_bytes_over_sector;

    // 要写入的最后一个字节，超过整扇区的部分（字节数）
    let end_bytes_over_sector = (file.file_off as usize + buff.len()).min(file.inode.i_size as usize) % constants::DISK_SECTOR_SIZE;

    // 申请单个扇区大小的缓冲区，用于循环读取扇区的数据
    let single_sector_buffer: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
    let mut succeed_bytes = 0usize;
    // 剩余要读取的字节数量
    let mut left_bytes = file.inode.i_size as i32 - file.file_off as i32;

    // 遍历所有的数据块扇区
    for block_idx in start_data_block_idx..=end_data_block_idx {
        // file.bytes_off所在的扇区，为相对扇区。从0开始
        let relative_block_idx = block_idx - start_data_block_idx;
        // 把缓冲区清空
        unsafe { single_sector_buffer.as_mut_ptr().write_bytes(0, single_sector_buffer.len()) };
        
        // 本次循环读取到的字节
        let mut bytes_read = 0; 
        // 要写入的数据扇区的LBA地址
        let data_block_lba = &mut file.inode.get_data_blocks()[block_idx];

        // 没有字节可以读取了
        if left_bytes <= 0 {
            break;
        }
        // 如果这个数据扇区没有地址
        if data_block_lba.is_empty() {
            continue;
        }
        // 读取出这个扇区
        disk.read_sectors(*data_block_lba, 1, single_sector_buffer);

        // 如果是第一个扇区，并且开始写入的字节开始偏移量不是整扇区
        if relative_block_idx == 0 && start_bytes_over_sector > 0 {
            bytes_read = start_bytes_away_sector.min(buff.len());
            // buff 的前半部分，使用读取到的扇区的后半部分代替
            buff[..bytes_read].copy_from_slice(&single_sector_buffer[start_bytes_over_sector..start_bytes_over_sector + bytes_read]);
        
        // 如果是操作最后一个扇区，并且写入的字节结束偏移量不是整扇区，并且不是第一个扇区
        } else if block_idx == end_data_block_idx && end_bytes_over_sector > 0 {
            // 如果这是最后一个扇区，同时也是第一个扇区
            let sector_start_idx = if block_idx == start_data_block_idx { start_bytes_over_sector } else {0};
            // 缓冲区要操作开始的字节下标
            let buf_start_idx = (constants::DISK_SECTOR_SIZE * relative_block_idx).overflowing_sub(start_bytes_over_sector);
            // 如果溢出了，那开始字节偏移就是0
            let buf_start_idx = if buf_start_idx.1 { 0 } else {buf_start_idx.0};
            // buff后半部分，使用读取到的扇区的前半部分代替
            buff[buf_start_idx..buf_start_idx+end_bytes_over_sector].copy_from_slice(&single_sector_buffer[sector_start_idx..end_bytes_over_sector]);
            bytes_read = end_bytes_over_sector;
        
        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            // 要操作的buff开始的字节偏移
            let buf_start_byte_idx = start_bytes_away_sector + (relative_block_idx.min(1) - 1) * constants::DISK_SECTOR_SIZE;
            // 要操作的buf结束的字节偏移
            let buf_end_byte_idx = start_bytes_away_sector + relative_block_idx * constants::DISK_SECTOR_SIZE;
            buff[buf_start_byte_idx .. buf_end_byte_idx].copy_from_slice(single_sector_buffer);
            bytes_read = single_sector_buffer.len();
        }
        succeed_bytes += bytes_read;
        left_bytes -= bytes_read as i32;
        file.file_off += bytes_read as u32;
    }
    return Result::Ok(succeed_bytes);
}


/**
 * 删除一个文件
 *   1. 删除这个文件的数据内容（inode）
 *   2. 删除这个文件所在父目录的目录项
 */
#[inline(never)]
pub fn remove_file(fs: &mut FileSystem, parent_inode: &mut OpenedInode, inode_to_remove: &OpenedInode) -> Result<(), FileError> {
    if inode_to_remove.open_cnts > 1 {
        return Result::Err(FileError::CouldNotRemoveAnOpenedFile);
    }

    // 1. 删除Inode
    inode::inode_remove(fs, inode_to_remove);

    // 2. 删除这个文件所在父目录的目录项
    let delete = dir_entry::remove_dir_entry(fs, parent_inode, DirEntrySearchReq::build().i_no(inode_to_remove.i_no));
    if !delete {
        return Result::Err(FileError::NotFound);
    }
    // 父目录操作完成后，保存到硬盘
    inode::sync_inode(fs, parent_inode);
    return Result::Ok(());
}