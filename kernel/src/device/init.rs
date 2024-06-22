use core::{borrow::{Borrow, BorrowMut}, mem::size_of, sync::atomic::AtomicU32};

use lazy_static::lazy_static;
use os_in_rust_common::{constants, linked_list::LinkedList, racy_cell::RacyCell, utils, ASSERT, printkln, sprintf};

use crate::{init, memory, println};
use crate::device::ata::Partition;

use super::{
    ata::{ATAChannel, ChannelIrqNoEnum, ChannelPortBaseEnum, Disk},
    drive::{BootSector, PartitionType},
};

const EMPTY_ATA_CHANNEL: ATAChannel = ATAChannel::empty();
/**
 * 全局用的2个ATA通道。下面挂载了硬盘
 */
pub static mut ALL_ATA_CHANNELS: [ATAChannel; 2] = [EMPTY_ATA_CHANNEL; 2];

/**
 * 分区列表
 */
pub static PARTITION_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

/**
 * 总扩展分区的LBA起始地址
 */
static mut MAIN_EXT_LBA_BASE: RacyCell<u32> = RacyCell::new(0);

/**
 * 获取该系统中的ATA Channel
 */
pub fn get_ata_channel(channel_idx: usize) -> &'static mut ATAChannel {
    unsafe { &mut ALL_ATA_CHANNELS[channel_idx] }
    // let all_ata_channels = unsafe { ALL_ATA_CHANNELS.get_mut() };
    // &mut all_ata_channels[channel_idx]
}

/**
 * 获取所有的分区列表
 */
pub fn get_all_partition() -> &'static mut LinkedList {
    unsafe { PARTITION_LIST.get_mut() }
}

/**
 * 总扩展分区的BLA起始地址
 */
fn main_extended_lba_base() -> &'static mut u32 {
    unsafe { MAIN_EXT_LBA_BASE.get_mut() }
}

/**
 * ATA通道初始化
 */
#[inline(never)]
pub fn ata_init() {
    // 读取内存，得到硬盘的数量
    let disk_cnt = unsafe { *(constants::DISK_LOCATION_IN_MEMORY as *const u8) };
    ASSERT!(disk_cnt > 0);
    // 通道的数量
    let channel_cnt = utils::div_ceil(disk_cnt, 2) as usize;
    let mut disk_start: u8 = 0;
    
    // 遍历每个通道
    for channel_idx in 0 .. channel_cnt {
        // 当前通道
        let channel = get_ata_channel(channel_idx);

        let (port_base, irq_no) = if channel_idx == 0 {
            (ChannelPortBaseEnum::Primary, ChannelIrqNoEnum::Primary)
        } else {
            (ChannelPortBaseEnum::Secondary, ChannelIrqNoEnum::Secondary)
        };
        // 初始化基本信息
        sprintf!(&mut channel.name, "ata{}", channel_idx);
        channel.port_base = port_base as u16;
        channel.irq_no = irq_no as u8;

        let channel_ptr = channel as *mut _;
        // 初始化该通道下的两个硬盘
        let disks = &mut channel.disks;
        for (disk_idx, disk) in disks.iter_mut().enumerate() {
            let disk_primary = disk_idx == 0;
            sprintf!(&mut disk.name, "sd{}", (b'a' + disk_start));
            disk.primary = disk_primary;
            disk.from_channel = channel_ptr;
            disk_start += 1;
            
            // 识别硬盘
            disk.identify();

            // 开始扫描该硬盘下的分区
            main_part_init(disk);
        }
    }
}

/**
 * 扫描该硬盘的主分区
 */
pub fn main_part_init(disk: &'static mut Disk) {
    let disk_ptr = disk as *const _;

    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) };

    // 读取该分区的第一个扇区，启动记录
    disk.read_sectors(0, 1, buf);
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
            // 填充该主分区的信息
            let primary_part = &mut disk.primary_parts[idx];
            let disk_name = core::str::from_utf8(&disk.name).expect("get ata channel name error");

            sprintf!(&mut primary_part.name, "{}{}",disk_name, idx);
            primary_part.lba_start = part_entry.start_lba;
            primary_part.sec_cnt = part_entry.sec_cnt;
            primary_part.from_disk = disk_ptr;
            
            // 放到队列中
            get_all_partition().append(&mut primary_part.tag);
            
            continue;
        }
        // 如果是扩展分区，那么需要进入扫描扩展分区

        // 全局设置总扩展分区的起始地址
        let main_extend_lba_base = main_extended_lba_base();
        *main_extend_lba_base = part_entry.start_lba;

        // 进入扩展分区的扫描。总逻辑分区LBA地址，逻辑分区号是0
        extended_part_init(disk, part_entry.start_lba.try_into().unwrap(), 0);
    }
}

/**
 * 扫描扩展分区
 *  - disk: 要扫描的硬盘
 *  - main_ext_lba: 总扩展分区的起始地址。所有子扩展分区的LBA地址都基于该地址
 *  - logic_part_no: 在该扩展分区中，逻辑分区起始的编号
 */
pub fn extended_part_init(disk: &mut Disk, main_ext_lba: usize, mut logic_part_no: usize) {
    
    let disk_ptr = disk as *const _;
    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) };
    // 读取该分区的第一个扇区，启动记录
    printkln!("read main ext");
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

            let logical_part = &mut disk.logical_parts[logic_part_no];
            let disk_name = core::str::from_utf8(&disk.name).expect("get ata channel name error");
            sprintf!(&mut logical_part.name, "{}{}", disk_name, idx);
            logical_part.lba_start = part_entry.start_lba;
            logical_part.sec_cnt = part_entry.sec_cnt;
            logical_part.from_disk = disk_ptr;
            
            logic_part_no += 1;
            
            // 把该逻辑分区加入队列
            get_all_partition().append(&mut logical_part.tag);
            
            continue;
        }

        // 扩展分区，递归扫描
        let main_extend_lba_base = main_extended_lba_base();
        extended_part_init(disk, (part_entry.start_lba + *main_extend_lba_base) as usize, logic_part_no);
    }
}
