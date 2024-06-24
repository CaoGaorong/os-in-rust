use core::{mem::{size_of, size_of_val}, slice};

use crate::device::ata::{Disk, Partition};

use super::superblock::SuperBlock;

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


pub fn install_filesystem(part: &mut Partition) {
    // 构建一个超级块出来。占用1个扇区
    let super_block = SuperBlock::new(part.abs_lba_start(0), part.sec_cnt);
    let disk = unsafe { &mut *part.from_disk };
    // 把超级块 写入到 该分区的
    disk.write_sector(unsafe { slice::from_raw_parts(&super_block as *const SuperBlock as *const _, size_of_val(&super_block)) }, part.abs_lba_start(1) as usize, 1)
}