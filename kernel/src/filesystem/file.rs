use core::mem::size_of;

use os_in_rust_common::{constants, printkln, ASSERT, MY_PANIC};


use crate::{filesystem::{dir_entry::FileType, global_file_table}, memory, thread};

use super::{
    dir_entry, file_descriptor::FileDescriptor, fs::{self, FileSystem}, inode::OpenedInode
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

    pub fn set_file_off(&mut self, off: u32) {
        self.file_off = off;
    }
}
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

}


pub fn open_file(file_path: &str, append: bool) -> Result<FileDescriptor, FileError>{
    if !file_path.starts_with("/") {
        return Result::Err(FileError::FilePathIllegal);
    }
    let fs = fs::get_filesystem();
    ASSERT!(fs.is_some());
    let fs = fs.unwrap();
    let fs = fs::get_filesystem();
    ASSERT!(fs.is_some());
    let fs = fs.unwrap();


    // 搜索到这个文件
    let searched_file = dir_entry::search_dir_entry(fs, file_path);
    if searched_file.is_none() {
        return Result::Err(FileError::NotFound);
    }
    let (searched_file, _) = searched_file.unwrap();

    // 如果打开的是一个文件夹，不允许打开
    if searched_file.file_type as FileType == FileType::Directory {
        return Result::Err(FileError::IsADirectory);
    }
    // 打开inode
    let file_inode = fs.inode_open(searched_file.i_no);
    
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
    ASSERT!(fs.is_some());
    let fs = fs.unwrap();
    let fs = fs::get_filesystem();
    ASSERT!(fs.is_some());
    let fs = fs.unwrap();


    let last_slash_idx = file_path.rfind("/");
    // 没有根目录，报错
    if !file_path.starts_with("/") || last_slash_idx.is_none() {
        return Result::Err(FileError::FilePathIllegal);
    }
    let last_slash_idx = last_slash_idx.unwrap();
    // 斜杠在最后一个字符，这说明是一个目录，也报错
    if last_slash_idx == file_path.len() - 1 {
        return Result::Err(FileError::IsADirectory);
    }
    // 该文件的目录路径
    let dir_path = &file_path[..last_slash_idx];
    // 该文件的名称
    let file_name = &file_path[last_slash_idx+1..];

    // 先搜索一下，目录是否存在
    let search_dir = dir_entry::search_dir_entry(fs, dir_path);
    if search_dir.is_none() {
        return Result::Err(FileError::ParentDirNotExists);
    }
    // 在搜索一下这个文件是否存在
    let dir_inode = search_dir.unwrap().1;
    let search_entry = dir_entry::do_search_dir_entry(fs, dir_inode, file_name);
    if search_entry.is_some() {
        return Result::Err(FileError::AlreadyExists);
    }

    let create_result = dir_entry::create_dir_entry(fs, dir_inode, file_name, FileType::Regular);
    // 创建目录项失败了
    if create_result.is_none() {
        return Result::Err(FileError::AlreadyExists);
    }
    let (_, opened_inode) = create_result.unwrap();
    

    // 得到一个打开文件
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
    // 要写入的数据大小
    let bytes = buff.len();

    let disk = unsafe { &mut *fs.base_part.from_disk };

    let start_data_block_idx = file.file_off as usize / constants::DISK_SECTOR_SIZE;
    // 要写入到文件的最后一个字节，所在该inode数据扇区的下标
    let end_data_block_idx = (file.file_off as usize + bytes) / constants::DISK_SECTOR_SIZE;
    // 如果涉及到间接块，那么需要加载间接块的数据
    if end_data_block_idx >= file.inode.get_direct_data_blocks_ref().len() {
        file.inode.load_data_block(fs);
    }

    // 要操作的文件偏移量，超过1个扇区的字节数
    let start_bytes_over_sector = file.file_off as usize % constants::DISK_SECTOR_SIZE;
    // 要操作的文件偏移量，相距首个扇区的距离
    let start_bytes_away_first_sector = constants::DISK_SECTOR_SIZE - start_bytes_over_sector;

    // 要写入的最后一个字节，超过整扇区的部分（字节数）
    let end_bytes_over_sector = (file.file_off as usize + bytes) % constants::DISK_SECTOR_SIZE;

    // 申请单个扇区大小的缓冲区，用于循环读取扇区的数据
    let single_sector_buffer: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
    let mut succeed_bytes = 0usize;

    // 遍历所有的数据块扇区
    for block_idx in start_data_block_idx..=end_data_block_idx {
        // 把缓冲区清空
        unsafe { single_sector_buffer.as_mut_ptr().write_bytes(0, single_sector_buffer.len()) };

        // 本次循环写入的字节数量
        let mut bytes_written = constants::DISK_SECTOR_SIZE;
        // 要写入的数据扇区的LBA地址
        let data_block_lba = &mut file.inode.get_data_blocks()[block_idx];

        // 如果这个数据扇区没有填充过，那么需要申请一个数据块
        if data_block_lba.is_empty() {
            *data_block_lba = fs.data_block_pool.apply_block(1);
        }

        // 如果是第一个扇区，并且开始写入的字节开始偏移量不是整扇区
        if succeed_bytes == 0 && start_bytes_over_sector > 0 {
            // 读取出这个扇区
            disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            // 该扇区的数据后半部分是要使用新数据代替
            bytes_written = start_bytes_away_first_sector.min(bytes);
            // 把缓冲区single_sector_buffer的后半部分，使用要写入的数据buff覆盖
            single_sector_buffer[start_bytes_over_sector..constants::DISK_SECTOR_SIZE.min(start_bytes_over_sector + bytes)].copy_from_slice(&buff[..bytes_written]);

        // 如果是操作最后一个扇区，并且写入的字节结束偏移量不是整扇区
        } else if block_idx == end_data_block_idx && end_bytes_over_sector > 0 {
            // 读取出这个扇区
            disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            bytes_written = end_bytes_over_sector;
            // 把single_sector_buffer的前半部分，使用要写入的buff数据代替
            single_sector_buffer[start_bytes_over_sector..end_bytes_over_sector].copy_from_slice(&buff[start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx.max(1) - 1)..=end_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx.max(1) - 1)]);

        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            let end_byte_idx = start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx - start_data_block_idx);
            // 如果buff跟扇区是对齐的，那么上面可能为0，所以这一设置一个最大值
            let end_byte_idx = end_byte_idx.max(constants::DISK_SECTOR_SIZE);
            single_sector_buffer.copy_from_slice(&buff[start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx.max(1) - 1) .. end_byte_idx]);
        }
        disk.write_sector(single_sector_buffer, *data_block_lba, 1);
        succeed_bytes += bytes_written;
    }
    memory::sys_free(single_sector_buffer.as_ptr() as usize);
    // 该文件操作的偏移量增加
    file.file_off += succeed_bytes as u32;
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
pub fn read_file(fs: &mut FileSystem, file: &mut OpenedFile, buff: &mut [u8], bytes: usize) {
    // 要写入的数据大小，取buff最小值和bytes最小值
    let bytes = bytes.min(buff.len());

    let disk = unsafe { &mut *fs.base_part.from_disk };

    let start_data_block_idx = file.file_off as usize / constants::DISK_SECTOR_SIZE;
    // 要写入到文件的最后一个字节，所在该inode数据扇区的下标
    let end_data_block_idx = (file.file_off as usize + bytes) / constants::DISK_SECTOR_SIZE;
    // 如果涉及到间接块，那么需要加载间接块的数据
    if end_data_block_idx >= file.inode.get_direct_data_blocks_ref().len() {
        file.inode.load_data_block(fs);
    }

    // 要操作的文件偏移量，超过1个扇区的字节数
    let start_bytes_over_sector = file.file_off as usize % constants::DISK_SECTOR_SIZE;

    // 要写入的最后一个字节，超过整扇区的部分（字节数）
    let end_bytes_over_sector = (file.file_off as usize + bytes) % constants::DISK_SECTOR_SIZE;

    // 申请单个扇区大小的缓冲区，用于循环读取扇区的数据
    let single_sector_buffer: &mut [u8; constants::DISK_SECTOR_SIZE] = memory::malloc(constants::DISK_SECTOR_SIZE);
    let mut succeed_bytes = 0usize;

    // 遍历所有的数据块扇区
    for block_idx in start_data_block_idx..=end_data_block_idx {
        // 把缓冲区清空
        unsafe { single_sector_buffer.as_mut_ptr().write_bytes(0, single_sector_buffer.len()) };
        
        // 本次循环读取到的字节
        let mut bytes_read = constants::DISK_SECTOR_SIZE; 
        // 要写入的数据扇区的LBA地址
        let data_block_lba = &mut file.inode.get_data_blocks()[block_idx];
        
        // 如果这个数据扇区没有地址，说明读取完成了
        if data_block_lba.is_empty() {
            break;
        }
        // 读取出这个扇区
        disk.read_sectors(*data_block_lba, 1, single_sector_buffer);

        // 如果是第一个扇区，并且开始写入的字节开始偏移量不是整扇区
        if succeed_bytes == 0 && start_bytes_over_sector > 0 {
            bytes_read = (constants::DISK_SECTOR_SIZE - start_bytes_over_sector).min(bytes);
            // buff 的前半部分，使用读取到的扇区的后半部分代替
            buff[..bytes_read].copy_from_slice(&single_sector_buffer[start_bytes_over_sector..constants::DISK_SECTOR_SIZE.min(bytes)]);
        
        // 如果是操作最后一个扇区，并且写入的字节结束偏移量不是整扇区，并且不是第一个扇区
        } else if block_idx == end_data_block_idx && end_bytes_over_sector > 0 && end_bytes_over_sector > start_bytes_over_sector {
            // 读取出这个扇区
            disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            // buff后半部分，使用读取到的扇区的前半部分代替
            buff[bytes - start_bytes_over_sector..].copy_from_slice(&single_sector_buffer[..end_bytes_over_sector]);
            bytes_read = end_bytes_over_sector;
        
        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            // buff起始地址，在扇区偏移中，剩下的部分
            let left_byte_off = constants::DISK_SECTOR_SIZE - start_bytes_over_sector;
            // 本次要操作结束的字节下标
            let end_byte_idx = left_byte_off + constants::DISK_SECTOR_SIZE * (block_idx - start_data_block_idx);
            // 如果buff跟扇区是对齐的，那么上面可能为0，所以这一设置一个最大值
            let end_byte_idx = end_byte_idx.max(constants::DISK_SECTOR_SIZE);
            buff[end_byte_idx - constants::DISK_SECTOR_SIZE .. end_byte_idx].copy_from_slice(single_sector_buffer);
        }
        disk.write_sector(single_sector_buffer, *data_block_lba, 1);
        succeed_bytes += bytes_read;
    }
}
