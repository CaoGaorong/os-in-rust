use core::{mem::{size_of, size_of_val}, slice};

use os_in_rust_common::{printkln, ASSERT};

use crate::device::{self, ata::{Disk, Partition}};

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
    let disk = unsafe { &mut *part.from_disk };
    // 把超级块 写入到 该分区的
    disk.write_sector(unsafe { slice::from_raw_parts(&super_block as *const SuperBlock as *const _, size_of_val(&super_block)) }, part.abs_lba_start(1) as usize, 1)
}