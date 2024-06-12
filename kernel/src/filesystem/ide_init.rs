use os_in_rust_common::{constants, utils, ASSERT};

pub fn ide_init() {
    // 读取内存，得到硬盘的数量
    let disk_cnt = unsafe { *(constants::DISK_LOCATION_IN_MEMORY as *const u8) }
    ASSERT!(disk_cnt > 0);
    // 通道的数量
    let channel_cnt = utils::div_ceil(disk_cnt, 2) as u8;

    
}