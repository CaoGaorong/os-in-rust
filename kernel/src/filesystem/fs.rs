use core::{arch::asm, mem::{size_of, size_of_val}, slice};

use os_in_rust_common::{constants, cstr_write, domain::{InodeNo, LbaAddr}, printkln, racy_cell::RacyCell, utils, ASSERT};

use crate::{device::{self, ata::Partition}, filesystem::dir::FileType};
use crate::memory;

use super::{dir::{DirEntry, MountedPartition}, inode::Inode, superblock::SuperBlock};

/**
 * 当前的挂载的分区
 */
static CUR_PARTITION: RacyCell<Option<MountedPartition>> = RacyCell::new(Option::None);

pub fn set_cur_part(cur_part: MountedPartition) {
    *unsafe { CUR_PARTITION.get_mut() } = Option::Some(cur_part);
}

pub fn get_cur_partition() -> Option<&'static MountedPartition> {
    unsafe { CUR_PARTITION.get_mut() }.as_ref()
}

#[inline(never)]
pub fn mount_part(part_name: &str) {
    printkln!("mount {}", part_name);
    // 找到所有分区
    let all_part = device::get_all_partition();
    // 遍历所有分区
    let target_part = all_part.iter()
        .for_each(|part_tag| {
            let part = Partition::parse_by_tag(part_tag);
            // printkln!("part:0x{:x}", part as *const _ as usize);
            // 找到这个分区了，设置为当前分区
            if part.get_name() == part_name {
                let disk = unsafe { &mut *part.from_disk };

                // SuperBlock
                let super_block: &mut SuperBlock = memory::malloc(size_of::<SuperBlock>());
                let sb_buf = unsafe { slice::from_raw_parts_mut(super_block as *mut _ as *mut u8, size_of::<SuperBlock>()) };
                // 读取SuperBlock
                disk.read_sectors(part.abs_lba_start(1), 1, sb_buf);


                // inode位图
                let inode_bitmap_len = super_block.inode_bitmap_lba.get_lba() as usize * constants::DISK_SECTOR_SIZE;
                let inode_bitmap_bits = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(inode_bitmap_len) as *mut u8, inode_bitmap_len) };
                disk.read_sectors(super_block.inode_bitmap_lba, super_block.inode_bitmap_secs as usize, inode_bitmap_bits);


                // 块位图
                let block_bitmap_len = super_block.block_bitmap_secs as usize * constants::DISK_SECTOR_SIZE;
                let block_bitmap_bits = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(block_bitmap_len) as *mut u8, inode_bitmap_len) };
                disk.read_sectors(super_block.block_bitmap_lba, super_block.block_bitmap_secs as usize, block_bitmap_bits);
                

                // 挂载的分区
                let mounted_part =  MountedPartition::new(part, super_block, inode_bitmap_bits, block_bitmap_bits);

                // 设置当前挂载的分区
                set_cur_part(mounted_part);
            }
        });
}

pub fn init() {
    // 取出primary通道
    let channel_idx = 0;
    let primary_channel = device::get_ata_channel(&channel_idx);
    let primary_channel = primary_channel.as_mut();
    ASSERT!(primary_channel.is_some());
    let primary_channel = primary_channel.unwrap();

    // 取出secondary disk: hd80M.img
    let secondary_disk = &mut primary_channel.disks[1];
    let secondary_disk = secondary_disk.as_mut();
    ASSERT!(secondary_disk.is_some());
    let secondary_disk =  secondary_disk.unwrap();

    // 取出第一个分区
    let first_part  = &mut secondary_disk.primary_parts[0];
    let first_part = first_part.as_mut();
    ASSERT!(first_part.is_some());
    let first_part = first_part.unwrap();

    // 把文件系统安装在第一个分区上
    install_filesystem(first_part);

}

/**
 * 安装文件系统
 * 我们文件系统的设计：
 * | 引导块(1扇区) | 超级块(1扇区) | inode位图(x扇区) | inode数组(y扇区)| 空闲数据块位图(z扇区)  | 根目录(1扇区) | 若干个数据块
 * 注意：这里根目录也属于数据块
 */
#[inline(never)]
#[no_mangle]
pub fn install_filesystem(part: &mut Partition) {
    
    // 安装superBlock
    let super_block = SuperBlock::new(part.abs_lba_start(0), part.sec_cnt);
    self::install_super_block(part, &super_block);

    // 先创建一个缓冲区，取三者的最大者
    let buff_max_secs = super_block.block_bitmap_secs
                        .max(super_block.inode_bitmap_secs)
                        .max(super_block.inode_table_secs);
    let buff_bytes = buff_max_secs as usize * constants::DISK_SECTOR_SIZE;
    let buff = unsafe { slice::from_raw_parts_mut(memory::sys_malloc(buff_bytes) as *mut u8, buff_bytes) };

    // 安装inode位图
    self::install_inode_bitmap(part, &super_block, buff);

    // 安装inode表（数组）
    self::install_inode_table(part, &super_block, buff);

    // 安装块位图
    self::install_block_bitmap(part, &super_block, buff);

    // 安装根目录
    self::install_root_dir(part, &super_block, buff);

    // 释放缓冲区
    memory::sys_free(buff.as_ptr() as usize);
}

/**
 * 在part分区中安装超级块super_block
 */
fn install_super_block(part: &mut Partition, super_block: &SuperBlock) {
    let disk = unsafe { &mut *part.from_disk };
    // 把超级块 写入到 该分区的
    disk.write_sector(unsafe { slice::from_raw_parts(super_block as *const SuperBlock as *const _, size_of_val(super_block)) }, part.abs_lba_start(1), 1);
}

/**
 * 在part分区中，安装空闲块位图
 */
#[inline(never)]
#[no_mangle]
fn install_block_bitmap(part: &mut Partition, super_block: &SuperBlock, buff: &mut [u8]) {
    // 块位图长度是扇区大小的倍数，当初是向上去整了的
    // 块位图中，位图的长度（按位计算）
    let block_bitmap_bit_len = super_block.block_bitmap_secs * constants::DISK_SECTOR_SIZE as u32 * 8;

    // 而实际有这么多个数据扇区（这才是块位图的有效长度bit）
    let data_block_secs = super_block.data_block_secs;
    
    // 无效的bit数量 = 位图长度(bits) - 实际数据扇区数量
    let invalid_bits = block_bitmap_bit_len - data_block_secs;

    // 我们把位图中用不到的位都设置为1，表示这些数据扇区不可用

    // 有效位长度是data_block_secs，后面的位都用不到
    // 先按照字节，把位图高字节设置为无效
    unsafe { buff.as_mut_ptr().add(utils::div_ceil(data_block_secs, 8) as usize).write_bytes(0xFF, invalid_bits as usize / 8); }

    // 最后一个有效字节，可能部分位是无效的。把高位的部分位，也设置为1
    for invalid_bit in  0 .. invalid_bits % 8 {
        buff[data_block_secs as usize / 8] |= 1 << (7 - invalid_bit);
    }

    // 位图的第0位设置为1，这一位是给根目录所在块的，设置为已占用
    buff[0] |= 0x01;

    // 块位图写入硬盘
    let disk = unsafe { &mut *part.from_disk };
    disk.write_sector(buff, super_block.block_bitmap_lba, super_block.block_bitmap_secs as usize);
}


/**
 * 安装inode位图
 */
#[inline(never)]
#[no_mangle]
fn install_inode_bitmap(part: &mut Partition, super_block: &SuperBlock, buff: &mut [u8]) {
    // 清零
    unsafe { buff.as_mut_ptr().write_bytes(0x00, buff.len()) };
    // buf[0] = 0b00000001
    // 表示位图的0号位是1，被根inode占用了
    buff[0] |= 0x01;

    // printkln!("install_inode_bitmap");
    let disk = unsafe { &mut *part.from_disk };
    disk.write_sector(buff, super_block.inode_bitmap_lba, super_block.inode_bitmap_secs as usize);
}


/**
 * 安装inode列表
 */
fn install_inode_table(part: &mut Partition, super_block: &SuperBlock, buff: &mut [u8]) {
    // 清零
    unsafe { buff.as_mut_ptr().write_bytes(0x00, buff.len()) };
    // 转成Inode数组
    let inode_table = unsafe { slice::from_raw_parts_mut(buff as *mut _ as *mut Inode, buff.len() / size_of::<Inode>()) };
    
    // 跟目录inode
    let root_inode = &mut inode_table[0];
    root_inode.i_no = InodeNo::new(0);
    root_inode.i_size = super_block.dir_entry_size * 2; // 2个目录：.和..
    // 根目录inode，数据区就是在第一个数据扇区
    root_inode.i_sectors[0] = super_block.data_lba_start;

    // 把inode列表写入到硬盘中
    let disk = unsafe { &mut *part.from_disk };
    disk.write_sector(buff, super_block.inode_table_lba, super_block.inode_table_secs as usize);
}

/**
 * 安装根目录项
 *  根目录有两项：.和..，都放在块的数据区
 */
fn install_root_dir(part: &mut Partition, super_block: &SuperBlock, buff: &mut [u8]) {
    // 清零
    unsafe { buff.as_mut_ptr().write_bytes(0x00, buff.len()) };
    // 转成目录项数组
    let dir_table = unsafe { slice::from_raw_parts_mut(buff as *mut _ as *mut DirEntry, buff.len() / size_of::<DirEntry>()) };
    {
        // . 目录项项
        let cur_dir = &mut dir_table[0];
        cstr_write!(&mut cur_dir.name, ".");
        cur_dir.i_no = InodeNo::from(0u32);
        cur_dir.file_type = FileType::Directory;
    }

    {
        // .. 目录项
        let last_dir = &mut dir_table[1];
        cstr_write!(&mut last_dir.name, "..");
        last_dir.i_no = InodeNo::from(0u32);
        last_dir.file_type = FileType::Directory;
    }
    // 把根目录的两个项：.和..，写入到数据扇区
    let disk = unsafe { &mut *part.from_disk };
    disk.write_sector(buff, super_block.data_lba_start, 1 as usize);

}