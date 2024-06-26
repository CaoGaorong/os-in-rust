use core::{arch::asm, mem::{size_of, size_of_val}, slice};

use os_in_rust_common::{constants, printkln, utils, ASSERT};

use crate::device::{self, ata::{Disk, Partition}};
use crate::memory;

use super::{inode::Inode, superblock::SuperBlock};

pub enum FileType {
    /**
     * 普通文件
     */
    Regular,
    /**
     * 目录
     */
    Directory,
    /**
     * 未知
     */
    Unknown,
}


pub fn init() {
    let channel_idx = 0;
    let primary_channel = device::get_ata_channel(&channel_idx);
    let primary_channel = primary_channel.as_mut();
    ASSERT!(primary_channel.is_some());
    let primary_channel = primary_channel.unwrap();
    let secondary_disk = &mut primary_channel.disks[1];
    let secondary_disk = secondary_disk.as_mut();
    ASSERT!(secondary_disk.is_some());
    let secondary_disk =  secondary_disk.unwrap();
    let first_part  = &mut secondary_disk.primary_parts[0];
    let first_part = first_part.as_mut();
    ASSERT!(first_part.is_some());
    let first_part = first_part.unwrap();

    install_filesystem(first_part);
}

#[inline(never)]
#[no_mangle]
pub fn install_filesystem(part: &mut Partition) {
    // printkln!("install file system");
    // 构建一个超级块出来。占用1个扇区
    let super_block = SuperBlock::new(part.abs_lba_start(0), part.sec_cnt);

    // 安装superBlock
    self::install_super_block(part, &super_block);

    // 先创建一个缓冲区，取三者的最大者
    let buff_max_secs = super_block.block_bitmap_secs
                        .max(super_block.inode_bitmap_secs)
                        .max(super_block.inode_table_secs);
    let buff_bytes = buff_max_secs as usize * constants::DISK_SECTOR_SIZE;
    // printkln!("buff bytes:{}", buff_bytes);
    let addr = memory::sys_malloc(buff_bytes);
    // printkln!("addr: 0x{:x}", addr);
    
    let buff = unsafe { slice::from_raw_parts_mut(addr as *mut u8, buff_bytes) };
    
    // printkln!("buffer len:{}", buff.len());
    printkln!("fuck");
    // 安装块位图
    self::install_block_bitmap(part, &super_block, buff);

    // 安装inode位图
    self::install_inode_bitmap(part, &super_block, buff);

    // 安装inode表（数组）
    self::install_inode_table(part, &super_block, buff);
}

/**
 * 在part分区中安装超级块super_block
 */
fn install_super_block(part: &mut Partition, super_block: &SuperBlock) {
    let disk = unsafe { &mut *part.from_disk };
    // 把超级块 写入到 该分区的
    disk.write_sector(unsafe { slice::from_raw_parts(super_block as *const SuperBlock as *const _, size_of_val(super_block)) }, part.abs_lba_start(1) as usize, 1);
}

/**
 * 在part分区中，安装块位图
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


    let disk = unsafe { &mut *part.from_disk };
    // printkln!("buf len:{}", buff.len());
    // printkln!("lba:{}, secs:{}", super_block.block_bitmap_lba, super_block.block_bitmap_secs);
    disk.write_sector(buff, super_block.block_bitmap_lba as usize, super_block.block_bitmap_secs as usize);
}

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
    disk.write_sector(buff, super_block.inode_bitmap_lba as usize, super_block.inode_bitmap_secs as usize);
}


fn install_inode_table(part: &mut Partition, super_block: &SuperBlock, buff: &mut [u8]) {
    // 清零
    unsafe { buff.as_mut_ptr().write_bytes(0x00, buff.len()) };
    // 转成Inode数组
    let inode_table = unsafe { slice::from_raw_parts_mut(buff as *mut _ as *mut Inode, buff.len() / size_of::<Inode>()) };
    
    // 跟目录inode
    let root_inode = &mut inode_table[0];
    root_inode.i_no = 0;
    root_inode.i_size = super_block.dir_entry_size * 2; // 2个目录：.和..
    // 根目录inode，数据区就是在第一个数据扇区
    root_inode.i_sectors[0] = Option::Some(super_block.data_lba_start);

    // 把inode列表写入到硬盘中
    let disk = unsafe { &mut *part.from_disk };
    disk.write_sector(buff, super_block.inode_table_lba as usize, super_block.inode_table_secs as usize);
}