
pub static LOADER_LBA:u32 = 2;
pub static LOADER_ADDR:u32 = 0x900;
pub static LOADER_SEC_CNT:u32 = 1;

pub static LOADER2_LBA: u32 = LOADER_LBA + LOADER_SEC_CNT;
pub const LOADER2_ADDR: u32 = 0xc00;
pub static LOADER2_SEC_CNT: u32 = 4;

pub static KERNEL_LBA:u32 = LOADER2_LBA + LOADER2_SEC_CNT;
pub static KERENL_ADDR:u32 = 0xc0001500;
pub static KERNEL_SEC_CNT: u32 = 200;

/**
 * 内核内存位图所在的地址
 */
pub const KERNEL_MEM_BITMAP_ADDR: u32 = 0xc009a000;
/**
 * 自定义开始的中断号
 */
pub static INTERRUPT_NO_START: u8 = 0x20;

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
