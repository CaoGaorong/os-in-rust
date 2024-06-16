use core::{mem::size_of, sync::atomic::AtomicU32};

use lazy_static::lazy_static;
use os_in_rust_common::{constants, linked_list::LinkedList, racy_cell::RacyCell, utils, ASSERT};

use crate::{init, memory};

use super::{
    ata::{ATAChannel, ChannelIrqNoEnum, ChannelPortBaseEnum, Disk},
    drive::{BootSector, PartitionType},
};

lazy_static! {
    /**
     * 全局用的2个ATA通道。下面挂载了硬盘
     */
    pub static ref ALL_ATA_CHANNELS: RacyCell<[ATAChannel; 2]> = RacyCell::new([ATAChannel::empty(); 2]);

    /**
     * 分区列表
     */
    pub static ref PARTITION_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

    

}

/**
 * 总扩展分区的LBA起始地址
 */
static mut MAIN_EXT_LBA_BASE: RacyCell<u32> = RacyCell::new(0);

/**
 * 获取该系统中的ATA Channel
 */
pub fn get_ata_channel(channel_idx: usize) -> &'static mut ATAChannel {
    let all_ata_channels = unsafe { ALL_ATA_CHANNELS.get_mut() };
    &mut all_ata_channels[channel_idx]
}

/**
 * 获取所有的分区列表
 */
pub fn get_all_partition() -> &LinkedList {
    unsafe { PARTITION_LIST.get_mut() }
}

/**
 * 总扩展分区的BLA起始地址
 */
fn main_extended_lba_base() -> &'static mut u32 {
    unsafe { MAIN_EXT_LBA_BASE.get_mut() }
}

pub fn ide_init() {
    // 读取内存，得到硬盘的数量
    let disk_cnt = unsafe { *(constants::DISK_LOCATION_IN_MEMORY as *const u8) };
    ASSERT!(disk_cnt > 0);
    // 通道的数量
    let channel_cnt = utils::div_ceil(disk_cnt, 2) as usize;
    let disk_start = usize;
    for channel_idx in 0 .. channel_cnt {
        let channel = get_ata_channel(channel_idx);
        let channel_name = concat!("ata", channel_idx);
        let (port_base, irq_no) = if channel_idx == 0 {
            (ChannelPortBaseEnum::Primary, ChannelIrqNoEnum::Primary)
        } else {
            (ChannelPortBaseEnum::Secondary, ChannelIrqNoEnum::Secondary)
        };
        // 初始化基本信息
        channel.init(name, port_base, irq_no);

        // 初始化该通道下的两个硬盘
        for (disk_idx, disk) in channel.disks.iter_mut().enumerate() {
            let disk_primary = disk_idx == 0;
            let disk_name = concat!("sd", (b'a' + disk_start) as char);
            disk_start += 1;
            disk.init(disk_name, disk_primary, channel);

            // 开始扫描该硬盘下的分区
            main_part_init(disk);
        }
    }
}

/**
 * 扫描该硬盘的主分区
 */
pub fn main_part_init(disk: &mut Disk) {
    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) }
    // 读取该分区的第一个扇区，启动记录
    disk.read_sectors(main_ext_lba, 1, buf);
    let boot_sector = unsafe { &*(boot_sec_addr as *const BootSector) };

    // 得到分区表
    let part_table = &boot_sector.part_table;

    for (idx, part_entry) in part_table.iter().enumerate() {
        // 空分区。忽略
        if part_entry.is_empty() {
            continue;
        }
        // 非扩展分区，是 有数据的主分区
        if !part_entry.is_extended() {
            let part_name = concat!(disk.get_name(), idx);
            // 填充该主分区的信息
            disk.primary_parts[idx].init(part_name, part_entry.start_lba, part_entry.sec_cnt, disk);
            // 放到队列中
            get_all_partition().append(&mut disk.primary_parts[idx].tag);
            continue;
        }
        // 如果是扩展分区，那么需要进入扫描扩展分区

        // 全局设置总扩展分区的起始地址
        let main_extend_lba_base = main_extended_lba_base();
        *main_extend_lba_base = part.start_lba;

        // 进入扩展分区的扫描。总逻辑分区LBA地址，逻辑分区号是0
        extended_part_init(disk, part.start_lba, 0);
    }
}

/**
 * 扫描扩展分区
 *  - disk: 要扫描的硬盘
 *  - main_ext_lba: 总扩展分区的起始地址。所有子扩展分区的LBA地址都基于该地址
 *  - logic_part_no: 在该扩展分区中，逻辑分区起始的编号
 */
pub fn extended_part_init(disk: &Disk, main_ext_lba: usize, logic_part_no: usize) {
    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) }
    // 读取该分区的第一个扇区，启动记录
    disk.read_sectors(main_ext_lba, 1, buf);
    let boot_sector = unsafe { &*(boot_sec_addr as *const BootSector) };

    // 得到分区表
    let part_table = &boot_sector.part_table;
    for (idx, part_entry) in part_table.iter().enumerate() {
        if part_entry.is_empty() {
            continue;
        }
        // 不是扩展分区，那么是真正有数据的逻辑分区
        if !part_entry.is_extended() {
            let part_name = concat!(disk.get_name(), logic_part_no);
            disk.logical_parts[logic_part_no].init(
                part_name,
                part_entry.start_lba,
                part_entry.sec_cnt,
                disk,
            );
            logic_part_no += 1;

            // 把该逻辑分区加入队列
            get_all_partition().append(&mut disk.logical_parts[logic_part_no].tag);
            continue;
        }

        // 扩展分区，递归扫描
        let main_extend_lba_base = main_extended_lba_base();
        extended_part_init(disk, part.start_lba + main_extend_lba_base, logic_part_no);
    }
}
