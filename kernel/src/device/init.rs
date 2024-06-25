use core::{borrow::{Borrow, BorrowMut}, ffi::CStr, mem::size_of, sync::atomic::AtomicU32};

use lazy_static::lazy_static;
use os_in_rust_common::{constants, cstr_write, cstring_utils, linked_list::LinkedList, printkln, racy_cell::RacyCell, utils, ASSERT};

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
const NONE_CHANNEL: Option<ATAChannel> = Option::None;
pub static mut ALL_ATA_CHANNELS: RacyCell<[Option<ATAChannel>; constants::ATA_CHANNEL_CNT]> = RacyCell::new([NONE_CHANNEL; constants::ATA_CHANNEL_CNT]);

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
pub fn get_ata_channel(channel_idx: &usize) -> &mut Option<ATAChannel> {
    let all_channel = unsafe { ALL_ATA_CHANNELS.get_mut() };
    let channel_opt = &mut all_channel[*channel_idx];
    return channel_opt;
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
    
    let mut buf = [0u8; 100];
    // 遍历每个通道
    for channel_idx in 0 .. channel_cnt {
        // 当前通道
        let mut channel = get_ata_channel(&channel_idx);

        let (port_base, irq_no) = if channel_idx == 0 {
            (ChannelPortBaseEnum::Primary, ChannelIrqNoEnum::Primary)
        } else {
            (ChannelPortBaseEnum::Secondary, ChannelIrqNoEnum::Secondary)
        };
        
        cstr_write!(&mut buf, "ata {}", channel_idx);

        *channel = Option::Some(ATAChannel::new(&buf, port_base, irq_no));
        let channel = channel.as_mut().unwrap();
        let channel_ptr = channel as *mut _;
        // 初始化该通道下的两个硬盘
        
        for disk_idx in 0 .. constants::DISK_CNT_PER_CHANNEL {
            cstr_write!(&mut buf, "sd{}", char::from_u32((b'a' + disk_start) as u32).unwrap());
            // printkln!("disk name:{}", cstring_utils::read_from_bytes(&buf).unwrap());
            let mut disk = &mut channel.disks[disk_idx];
            {
                *disk = Option::Some(Disk::new(&buf, channel_ptr, disk_idx == 0));
            }
            match disk {
                None => continue,
                Some(d) => {
                    disk_start += 1;
                
                    // 识别硬盘
                    d.identify();
    
                    // 开始扫描该硬盘下的分区
                    main_part_init(d);
                },
            }
        }
    }
}

/**
 * 扫描该硬盘的主分区
 */
pub fn main_part_init(disk: &mut Disk) {
    let disk_ptr = disk as *mut _;

    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) };

    // 读取该分区的第一个扇区，启动记录
    disk.read_sectors(0, 1, buf);
    let boot_sector = unsafe { &*(boot_sec_addr as *const BootSector) };

    // 得到分区表
    let part_table = &boot_sector.part_table;
    let mut buf = [0u8; 100];
    for (idx, part_entry) in part_table.iter().enumerate() {
        // 空分区。忽略
        if part_entry.is_empty() {
            continue;
        }
        // 非扩展分区，是 有数据的主分区
        if !part_entry.is_extended() {
            // 读取disk名称
            let disk_name = cstring_utils::read_from_bytes(&disk.name);
            ASSERT!(disk_name.is_some());
            // 该分区名称 = 磁盘名称 + i
            cstr_write!(&mut buf, "{}{}", disk_name.unwrap(), idx);

            // 填充该主分区的信息
            let mut primary_part = &mut disk.primary_parts[idx];
            *primary_part = Option::Some(Partition::new(&buf, part_entry.start_lba, part_entry.sec_cnt, disk_ptr));
            
            // 放到队列中
            get_all_partition().append(&mut primary_part.as_mut().unwrap().tag);
            
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
    
    let disk_ptr = disk as *mut _;
    // 申请内存。为了防止栈溢出，因此不使用局部变量
    let boot_sec_addr = memory::sys_malloc(size_of::<BootSector>());
    let buf = unsafe { core::slice::from_raw_parts_mut(boot_sec_addr as *mut u8, size_of::<BootSector>()) };
    // 读取该分区的第一个扇区，启动记录
    disk.read_sectors(main_ext_lba, 1, buf);
    let boot_sector = unsafe { &*(boot_sec_addr as *const BootSector) };

    let mut buf = [0u8; 100];
    // 得到分区表
    let part_table = &boot_sector.part_table;
    for (idx, part_entry) in part_table.iter().enumerate() {
        if part_entry.is_empty() {
            continue;
        }
        // 不是扩展分区，那么是真正有数据的逻辑分区
        if !part_entry.is_extended() {
            // 得到磁盘名称
            let disk_name = cstring_utils::read_from_bytes(&disk.name);
            ASSERT!(disk_name.is_some());
            // 该分区名称 = 磁盘名称 + i
            cstr_write!(&mut buf, "{}{}", disk_name.unwrap(), logic_part_no + 4);
            
            // 填充分区信息
            let mut logical_part = &mut disk.logical_parts[logic_part_no];
            logic_part_no += 1;
            
            *logical_part = Option::Some(Partition::new(&buf, main_ext_lba as u32 + part_entry.start_lba, part_entry.sec_cnt, disk_ptr));
            
            let part = logical_part.as_mut().unwrap();
            // 把该逻辑分区加入队列
            get_all_partition().append(&mut part.tag);
            continue;
        }

        // 扩展分区，递归扫描
        let main_extend_lba_base = main_extended_lba_base();
        extended_part_init(disk, (part_entry.start_lba + *main_extend_lba_base) as usize, logic_part_no);
    }
}
