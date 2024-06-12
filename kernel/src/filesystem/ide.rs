use core::slice;

use os_in_rust_common::{bitmap::BitMap, constants, linked_list::{LinkedList, LinkedNode}};

use crate::sync::{Lock, Semaphore};

/**
 * 本文件是关于IDE硬盘的通道的相关结构定义
 * 可以见 <https://wiki.osdev.org/ATA_PIO_Mode>
 * 
 */

/**
 * ide通道的逻辑结构
 * 关于ide通道，寄存器可以见：<https://wiki.osdev.org/ATA_PIO_Mode#Registers>
 * 
 * 一个机器可以挂在2个通道（主从），每个通道可以挂在2个硬盘（主从）。每个通道占据一个8259的中断端口
 */
struct IdeChannel {
    /**
     * IDE通道名称
     */
    name: [u8; constants::IDE_CHANNEL_NAME_LEN],
    /**
     * 本通道的起始端口号
     * 通道用到的硬盘控制器的寄存器，都是根据这个起始端口号递增的，
     * 详情见：<https://wiki.osdev.org/ATA_PIO_Mode#Registers>
     */
    port_base: u16,

    /**
     * 该通道用的中断号。1个通道占据一个中断端口，有一个中断号
     */
    irq_no: u8, 

    /**
     * 是否正在等待中断
     */
    expecting_intr: bool,

    /**
     * 同步锁
     */
    lock: Lock,

    /**
     * 使用信号量阻塞自己。当硬盘完成中断后产生的中断唤醒自己
     */
    disk_done: Semaphore,

    /**
     * 一个通道可以挂在两个硬盘
     */
    disks: [Disk; 2],
}

/**
 * 一个硬盘的结构
 */
pub struct Disk {
    /**
     * 硬盘的名称
     */
    name: [u8; constants::DISK_NAME_LEN],
    /**
     * 该硬盘归属的通道
     */
    from_channel: Option<&'static IdeChannel>,

    /**
     * 是否是主硬盘
     */
    primary: bool,

    /**
     * 一个硬盘最多4个主分区
     */
    primary_parts: [Partition; 4],

    /**
     * 逻辑分区。理论上一个硬盘无限多个逻辑分区数量
     */
    logical_parts: [Partition; constants::DISK_LOGICAL_PARTITION_CNT]
}

/**
 * 硬盘中的分区结构（逻辑结构）
 */
pub struct Partition {
    /**
     * 分区名称
     */
    name: [u8; constants::DISK_NAME_LEN],
    /**
     * 该分区位于硬盘的起始扇区数
     */
    lba_start: u32, 
    /**
     * 该分区占用的扇区数量
     */
    sec_cnt: u32,
    /**
     * 该分区归属的硬盘
     */
    from_disk: Option<&'static Disk>,
    /**
     * 组成链表的tag
     */
    tag: LinkedNode,

    /**
     * 本硬盘的超级块的地址
     */
    super_block: Option<&'static SuperBlock>,

    /**
     * 块位图
     */
    block_bitmap: BitMap, 
    /**
     * 节点位图
     */
    inode_bitmap: BitMap, 

    /**
     * 该硬盘打开的inode节点队列
     */
    open_inodes: LinkedList,
}
pub struct SuperBlock {

}

/**
 * 通道起始端口号
 */
pub enum ChannelPortBaseEnum {
    Primary = 0x1f0,
    Secondary = 0x170,
}
/**
 * 通道请求端口号。8259A设定的起始端口号 + 14/15
 *  这里14和15是8259A的这个端口位置。看下面Primary ATA和 Secondary ATA
 *                     ____________                          ____________
 * Real Time Clock --> |            |   Timer -------------> |            |
 * ACPI -------------> |            |   Keyboard-----------> |            |      _____
 * Available --------> | Secondary  |----------------------> | Primary    |     |     |
 * Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
 * Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
 * Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
 * Primary ATA ------> |            |   Floppy disk -------> |            |
 * Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
*/
pub enum ChannelIrqNoEnum {
    Primary = constants::INTERRUPT_NO_START + 14,
    Secondary = constants::INTERRUPT_NO_START + 15,
}

impl IdeChannel {

    /**
     * 初始化构建一个通道
     *  - name: 通道名称
     *  - port_base: 通道起始
     */
    pub const fn new(name: &str, port_base: ChannelPortBaseEnum, irq_no: ChannelIrqNoEnum, disks: [Disk; 2]) -> Self {
        Self {
            name: [u8; constants::IDE_CHANNEL_NAME_LEN].copy_from_slice(name),
            port_base: port_base as u16,
            irq_no: irq_no as u8,
            expecting_intr: false,
            lock: Lock::new(),
            disk_done: Semaphore::new(0),
            disks,
        }
    }

    /**
     * 该ide通道的寄存器端口号，都基于port_base
     * 可以见<https://wiki.osdev.org/ATA_PIO_Mode#Registers>
     */
    pub fn data_register_port(&self) -> u16 {
        self.port_base
    }
    pub fn error_register_port(&self) -> u16 {
        self.port_base + 1
    }
    pub fn sec_cnt_port(&self) -> u16 {
        self.port_base + 2
    }
    pub fn lba_low_port(&self) -> u16 {
        self.port_base + 3
    }
    pub fn lba_middle_port(&self) -> u16 {
        self.port_base + 4
    }
    pub fn lba_hight_port(&self) -> u16 {
        self.port_base + 5
    }
    pub fn dev_register_port(&self) -> u16 {
        self.port_base + 6
    }
    pub fn status_register_port(&self) -> u16 {
        self.port_base + 7
    }
    pub fn command_register_port(&self) -> u16 {
        self.status_register_port()
    }
    pub fn alt_status_register_port(&self) -> u16 {
        self.port_base + 0x206
    }
    
    pub fn alt_ctrl_register_port(&self) -> u16 {
        self.alt_status_register_port()
    }
}

impl Disk {
    pub const fn new(name: &str, primary: bool, primary_parts: [Partition; 4], logical_parts: [Partition; constants::DISK_LOGICAL_PARTITION_CNT]) -> Self {
        Self {
            name: [0; constants::DISK_NAME_LEN].copy_from_slice(name),
            from_channel: Option::None,
            primary,
            primary_parts: primary_parts,
            logical_parts: logical_parts,
        }
    }
}

impl Partition {
    pub const fn new(name: &str, lba_start: u32, sec_cnt: u32, ) -> Self {
        Self {
            name: [u8; constants::DISK_NAME_LEN],
            lba_start: lba_start,
            sec_cnt: sec_cnt,
            from_disk: Option::None,
            tag: LinkedNode::new(),
            super_block: Option::None,
            block_bitmap: BitMap::empty(),
            inode_bitmap: BitMap::empty(),
            open_inodes: LinkedList::new(),
        }
    }
}