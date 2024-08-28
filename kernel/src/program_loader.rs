use os_in_rust_common::{constants, domain::LbaAddr, utils, ASSERT, MY_PANIC};

use crate::{device, filesystem::File, memory};


/**
 * 同步用户程序。
 * 把裸盘中的用户程序读取出来，然后写入到文件系统中。
 *     file_lba：用户程序文件所在的裸盘的LBA地址
 *     file_size: 这个用户程序的大小。单位字节
 *     file_path_to_sync: 要写入文件系统的路径
 */
#[inline(never)]
pub fn sync_program(file_lba: LbaAddr, file_size: usize, file_path_to_sync: &str) {
    // 文件占用的扇区数量
    let sec_cnt = utils::div_ceil(file_size as u32, constants::DISK_SECTOR_SIZE as u32) as usize;
    
    // 主channel
    let channel_idx = 0;
    let channel = device::get_ata_channel(&channel_idx);
    ASSERT!(channel.is_some());
    let channel = channel.as_mut();
    let channel = channel.unwrap();

    // 主硬盘
    let disk = channel.disks[0].as_mut();
    let disk = disk.unwrap();

    // 创建一个缓冲区
    let buff = unsafe { core::slice::from_raw_parts_mut(memory::sys_malloc(sec_cnt * constants::DISK_SECTOR_SIZE) as *mut u8, sec_cnt * constants::DISK_SECTOR_SIZE) };
    
    // 把这个文件从缓冲区读取出来
    disk.read_sectors(file_lba, sec_cnt, buff);

    
    // 创建这个文件
    let file = File::create(file_path_to_sync);
    if file.is_err() {
        MY_PANIC!("failed to create file. error: {:?}", file.unwrap_err());
        return;
    }
    let mut file = file.unwrap();

    // 写入文件
    let res = file.write(buff);
    ASSERT!(res.is_ok());

    // 释放缓冲区
    memory::sys_free(buff.as_ptr() as usize);
}
