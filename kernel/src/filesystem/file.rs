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
    fs::FileSystem,
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

/**
* 把buff数组写入到file文件中，写入bytes个字节
*
           first_sector
       LBA  0             1              2             3
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
        let bytes_written:usize;
        // 要写入的数据扇区的LBA地址
        let data_block_lba = &mut file.inode.get_data_blocks()[block_idx];
        
        // 如果这个数据扇区没有填充过，那么需要申请一个数据块
        if data_block_lba.is_empty() {
            *data_block_lba = fs.data_block_pool.apply_block(1);
            bytes_written = constants::DISK_SECTOR_SIZE;
        }

        // 如果是第一个扇区，并且开始写入的字节开始偏移量不是整扇区
        if succeed_bytes == 0 && start_bytes_over_sector > 0 {
            disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            bytes_written = (constants::DISK_SECTOR_SIZE - start_bytes_over_sector).min(bytes);
            single_sector_buffer[start_bytes_over_sector..].copy_from_slice(&buff[..bytes_written]);
        
        // 如果是操作最后一个扇区，并且写入的字节结束偏移量不是整扇区，并且不是第一个扇区
        } else if block_idx == end_data_block_idx && end_bytes_over_sector > 0 && end_bytes_over_sector > start_bytes_over_sector {
            disk.read_sectors(*data_block_lba, 1, single_sector_buffer);
            bytes_written = end_bytes_over_sector;
            single_sector_buffer[..bytes_written].copy_from_slice(&buff[bytes - start_bytes_over_sector..]);
        
        // 不是第一个扇区，也不是最后一个扇区，那就是中间连续的扇区，直接复制就好
        } else {
            let start_byte_idx = start_bytes_over_sector + constants::DISK_SECTOR_SIZE * block_idx;
            single_sector_buffer.copy_from_slice(&buff[start_byte_idx.. start_byte_idx + constants::DISK_SECTOR_SIZE]);
            bytes_written = constants::DISK_SECTOR_SIZE;
        }
        disk.write_sector(single_sector_buffer, *data_block_lba, 1);
        succeed_bytes += bytes_written;
    }
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
pub fn read_file(fs: &mut FileSystem, file: &mut OpenedFile, buff: &mut [u8], bytes: usize) {
    // 要写入的数据大小，取buff最小值和bytes最小值
    let bytes = bytes.min(buff.len());

    let disk = unsafe { &mut *fs.base_part.from_disk };

    // 要读取的数据，所在开始的该inode数据块列表下标
    let start_data_block_idx = file.file_off as usize / constants::DISK_SECTOR_SIZE;
    // 要读取的数据，所在结束的该inode数据块列表下标
    let end_data_block_idx = (file.file_off as usize + bytes) / constants::DISK_SECTOR_SIZE;

    // 如果涉及到间接块，那么需要加载间接块的数据
    if end_data_block_idx >= file.inode.get_direct_data_blocks_ref().len() {
        file.inode.load_data_block(fs);
    }

    let start_bytes_over_sector = file.file_off as usize % constants::DISK_SECTOR_SIZE;
    // 说明要读取的开始的字节，不是从一个完整的扇区开始
    if start_bytes_over_sector > 0 {
        // 专门用去存储首个扇区的数据
        let first_data_sectors: &mut [u8; constants::DISK_SECTOR_SIZE] =
            memory::malloc(constants::DISK_SECTOR_SIZE);
        // 要读取的首个扇区的LBA地址
        let start_data_block_lba = file.inode.get_data_blocks_ref()[start_data_block_idx];
        // 开始读取首个扇区的数据
        disk.read_sectors(start_data_block_lba, 1, first_data_sectors);
        // 把首个扇区的后半部分数据，拷贝到buff数组的前半部分中
        buff[..constants::DISK_SECTOR_SIZE - start_bytes_over_sector].copy_from_slice(
            &first_data_sectors[start_bytes_over_sector..first_data_sectors.len()],
        );
        memory::sys_free(first_data_sectors.as_ptr() as usize);
    }
    let end_bytes_over_sectors = (file.file_off as usize + bytes) % constants::DISK_SECTOR_SIZE;
    if end_bytes_over_sectors > 0 && end_bytes_over_sectors > start_bytes_over_sector {
        // 专门用去存储最后个扇区的数据
        let last_data_sectors: &mut [u8; constants::DISK_SECTOR_SIZE] =
            memory::malloc(constants::DISK_SECTOR_SIZE);
        // 要读取的最后一个扇区的LBA地址
        let last_data_block_lba = file.inode.get_data_blocks_ref()[end_data_block_idx];
        // 开始读取最后一个扇区的数据
        disk.read_sectors(last_data_block_lba, 1, last_data_sectors);
        // 最后一个扇区的前半部分数据，拷贝到buff数组的后半部分中
        buff[bytes - end_bytes_over_sectors..]
            .copy_from_slice(&last_data_sectors[..end_bytes_over_sectors]);
    }

    // 要读取信息是完整的开始数据扇区的下标
    let full_block_start_idx = if start_bytes_over_sector > 0 {
        start_data_block_idx + 1
    } else {
        start_data_block_idx
    };
    // 要读取信息是完整的结束数据扇区的下标
    let full_block_end_idx =
        if end_bytes_over_sectors > 0 && end_bytes_over_sectors > start_bytes_over_sector {
            end_data_block_idx - 1
        } else {
            end_data_block_idx
        };
    // 遍历要读取的数据块
    for data_block_idx in full_block_start_idx..=full_block_end_idx {
        // 找到要读取的数据块LBA
        let data_block_lba = file.inode.get_data_blocks_ref()[data_block_idx];
        // 从硬盘中加载出数据
        disk.read_sectors(
            data_block_lba,
            1,
            &mut buff[(data_block_idx - start_data_block_idx) * constants::DISK_SECTOR_SIZE
                + (constants::DISK_SECTOR_SIZE - start_bytes_over_sector)..],
        )
    }
}
