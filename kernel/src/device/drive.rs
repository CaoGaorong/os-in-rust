use os_in_rust_common::domain::LbaAddr;

/**
 * 分区相关
 */

/**
 * MBR或者EBR所在引导扇区的物理结构
 */
 #[repr(C, packed)]
pub struct BootSector {
    /**
     * MBR的代码
     */
    boot_record: [u8; 446],
    /**
     * 4个分区表项
     */
    pub part_table: [PartitionTableEntry; 4],
    /**
     * 魔数0x55, 0xaa
     */
    sign: u16,
}

/**
 * 分区表项的物理结构
 * <https://wiki.osdev.org/Partition_Table>
 */
#[repr(C, packed)]
pub struct PartitionTableEntry {
    /**
     * 0x0: 不可引导
     * 0x80: 可以引导
     */
    pub boot_indicator: u8,
    /**
     * 磁头号
     */
    starting_head: u8,
    /**
     * 扇区号。
     * [0, 6) bit是扇区号
     * [6, 8) bit 是柱面号
     */
    starting_sector: u8,
    /**
     * 柱面号。
     * 柱面号一共10位，这里8位 + 上面2位
     */
    starting_cylinder: u8,
    /**
     * 系统id的类型
     * @see PartitionType
     */
    pub part_type: u8,
    /**
     * 结束的磁头号
     */
    ending_head: u8,
    /**
     * 结束的扇区号
     */
    ending_sector: u8,
    /**
     * 结束的柱面号
     */
    ending_cylinder: u8,
    
    /**
     * 该分区所在的扇区偏移量（起始LBA地址）
     */
    pub start_lba: LbaAddr,
    /**
     * 该分区占用的扇区数量
     */
    pub sec_cnt: u32,
}
impl PartitionTableEntry {
    /**
     * 是不是扩展分区
     */
    pub fn is_extended(&self) -> bool {
        self.part_type == PartitionType::Extended as u8
    }
    /**
     * 如果是0，那么这个分区就是空的，不存在
     */
    pub fn is_empty(&self) -> bool {
        self.part_type == 0x0
    }
}


pub enum PartitionType {
    // 扩展类型
    Extended = 0x5,
}


