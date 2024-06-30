
pub static LOADER_LBA:u32 = 2;
pub static LOADER_ADDR:u32 = 0x900;
pub static LOADER_SEC_CNT:u32 = 1;

pub static LOADER2_LBA: u32 = LOADER_LBA + LOADER_SEC_CNT;
pub const LOADER2_ADDR: u32 = 0xc00;
pub static LOADER2_SEC_CNT: u32 = 4;

pub static KERNEL_LBA:u32 = LOADER2_LBA + LOADER2_SEC_CNT;
pub const KERNEL_ADDR:u32 = 0xc0001500;
pub static KERNEL_SEC_CNT: u32 = 200;


/**
 * GDT元素个数
 */
pub const GDT_SIZE: usize = 7;

/**
 * 内核内存位图所在的地址
 */
pub const KERNEL_MEM_BITMAP_ADDR: u32 = 0xc009a000;
/**
 * 自定义开始的中断号
 */
pub const INTERRUPT_NO_START: u8 = 0x20;

/**
 * 计数器的脉冲频率（8253的默认值）
 */
pub const PIT_DEFAULT_FREQUENCY: u32 = 1193180;

/**
 * 计数器最大递减的值（8253的默认值）
 */
pub const PIC_MAX_DECREMENT: u32 = 65536;

/**
 * 本系统设置的时钟中断的频率
 */
pub const TIMER_INTR_FREQUENCY: u16 = 100;


/**
 * 页表项的数量
 */
pub const PAGE_TABLE_ENTRY_COUNT: usize = 1024;


/**
 * 内核页目录表的地址
 */
pub const KERNEL_PAGE_DIR_ADDR: usize = 0x100000;


/**
 * 内核页表的地址
 */
pub const KERNEL_PAGE_TABLE_ADDR: usize = 0x100000 + 0x1000;


/**
 * 页的大小：1024
 */
pub const PAGE_SIZE: u32 = 4 * 1024;

/**
 * 实模式的内存大小
 */
pub const REAL_MEMORY: usize = 0x100000;

/**
 * 内核开始的内存
 * 内核占4G的高1G。也就是4GB * 3/4
 */
pub const KERNEL_ADDR_START: usize = (4 as u64 * 1024 * 1024 * 1024 / 4 * 3) as usize;

/**
 * 内核页目录表，开始的页目录项的下标
 * 内核占4G的高1G，也就是1024占3/4。也就是从页目录表的第768项开始
 */
pub const KERNEL_DIR_ENTRY_START_IDX: usize = 1024 / 4 * 3;

/**
 * 内核页目录表最后一项的下标。
 * 一共1024项，最后一项下标就是1023
 */
pub const  KERNEL_DIR_ENTRY_END_IDX: usize = 1024 - 1;

/**
 * 内核页表的数量：255个
 * dir[0] = table[0]
 * dir[768] = table[0]
 * dir[769] = table[1]
 * ......
 * dir[1022] = table[254]
 */
pub const  KERNEL_PAGE_TABLE_CNT: usize = KERNEL_DIR_ENTRY_END_IDX - KERNEL_DIR_ENTRY_START_IDX;

pub const TASK_STRUCT_STACK_MAGIC: u32 = 0x20010217;

pub const MAIN_THREAD_NAME: &str = "main";

/** 
 * 任务的默认优先级。也就是可用的ticks
*/
pub const TASK_DEFAULT_PRIORITY: u8 = 5;


pub const KEYBOARD_KEY_COUNT: usize = 0x3B;

/**
 * 用户进程的堆内存起始地址
 */
pub const USER_PROCESS_ADDR_START: usize = 0x8048000;

/**
 * 用户进程的栈地址(虚拟地址)
 */
pub const USER_STACK_BASE_ADDR: usize = 0xc0000000;

/**
 * 用户进程的栈地址(虚拟地址)
 */
pub const USER_STACK_TOP_ADDR: usize = USER_STACK_BASE_ADDR - 0x1000;

/**
 * 系统调用的函数数量
 */
pub const SYSTEM_CALL_HANDLER_CNT: usize = 100;


/**
 * 内存块容器的规格种类。7种
 */
pub const MEM_BLOCK_CONTAINER_CNT: usize = 7;

/**
 * 最小的内存块的大小
 */
pub const MINIMAL_BLOCK_SIZE: usize = 16;


/**
 * IDE通道名称的长度。10个字节
 */
pub const IDE_CHANNEL_NAME_LEN: usize = 10;

pub const DISK_NAME_LEN: usize = 10;

/**
 * ATA通道的数量
 */
pub const ATA_CHANNEL_CNT: usize = 2;
/**
 * 每个通道的磁盘数量
 */
pub const DISK_CNT_PER_CHANNEL: usize = 2;

/**
 * 每个硬盘主分区的数量
 */
pub const DISK_PRIMARY_PARTITION_CNT: usize = 4;

/**
 * 硬盘逻辑分区的数量限制。理论上一个硬盘无限个逻辑分区，但是这里还是要限制一下
 */
pub const DISK_LOGICAL_PARTITION_CNT: usize = 10;

/**
 * 硬盘的数量。BIOS把硬盘数量写入到了这个地址
 */
pub const DISK_LOCATION_IN_MEMORY: usize = 0x475;

/**
 * 硬盘的最大容量（单位字节）80MB
 */
pub const DISK_MAX_SIZE: u64 = 80 * 1024 * 1024;

/**
 * 硬盘中一个扇区的大小。512字节
 */
pub const DISK_SECTOR_SIZE: usize = 512;

/**
 * idle线程的名称
 */
pub static IDLE_THREAD_NAME: &str = "idle";


/**
 * 一个进程最大打开文件的数量
 */
pub const MAX_FILES_PER_PROC: usize = 8;