use core::mem::size_of;

use os_in_rust_common::{constants, domain::InodeNo, racy_cell::RacyCell, utils, ASSERT, MY_PANIC};

use crate::{
    filesystem::{
        dir::{self, DirEntry, FileType},
        global_file_table,
    },
    init, memory, thread,
};

use super::{
    dir::Dir,
    fs::{self, FileSystem},
    inode::{Inode, OpenedInode},
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
    /**
     * 这个打开的文件的操作标识
     */
    flag: FileFlag,
}

impl OpenedFile {
    pub fn new(inode: &'static mut OpenedInode) -> Self {
        let file_size = inode.i_size;
        Self {
            inode,
            file_off: file_size,
            flag: FileFlag::Init,
        }
    }
}
pub enum FileFlag {
    Init,
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
#[inline(never)]
pub fn write_files(file_path: &str, buff: &[u8], bytes: usize) {
    let fs = fs::get_filesystem();
    ASSERT!(fs.is_some());
    let fs = fs.unwrap();
    let searched_file = dir::search_file(fs, file_path);
    ASSERT!(searched_file.is_ok());
    let searched_file = searched_file.unwrap();
    let file_inode = fs.inode_open(searched_file.i_no);
    let mut opened_file = OpenedFile::new(file_inode);
    self::write_file(fs, &mut opened_file, buff, bytes);
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
pub fn write_file(fs: &mut FileSystem, file: &mut OpenedFile, buff: &[u8], bytes: usize) {
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
    // 要操作的文件偏移量，相距首个扇区的距离
    let start_bytes_away_first_sector = constants::DISK_SECTOR_SIZE - start_bytes_over_sector;

    // 要写入的最后一个字节，超过整扇区的部分（字节数）
    let end_bytes_over_sector = (file.file_off as usize + bytes) % constants::DISK_SECTOR_SIZE;
    let end_bytes_away_last_sector = constants::DISK_SECTOR_SIZE - end_bytes_over_sector;

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
            single_sector_buffer[..end_bytes_over_sector].copy_from_slice(&buff[start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx - 1).max(0)..]);

        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            let end_byte_idx = start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx - start_data_block_idx);
            // 如果buff跟扇区是对齐的，那么上面可能为0，所以这一设置一个最大值
            let end_byte_idx = end_byte_idx.max(constants::DISK_SECTOR_SIZE);
            single_sector_buffer.copy_from_slice(&buff[start_bytes_over_sector + constants::DISK_SECTOR_SIZE * (block_idx - 1).max(0) .. end_byte_idx]);
        }
        disk.write_sector(single_sector_buffer, *data_block_lba, 1);
        succeed_bytes += bytes_written;
    }
    memory::sys_free(single_sector_buffer.as_ptr() as usize);
    // 该文件操作的偏移量增加
    file.file_off += succeed_bytes as u32;

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
