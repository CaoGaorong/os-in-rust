
pub static LOADER_LBA:u32 = 2;
pub static LOADER_ADDR:u32 = 0x900;
pub static LOADER_SEC_CNT:u32 = 1;

pub static LOADER2_LBA: u32 = LOADER_LBA + LOADER_SEC_CNT;
pub static LOADER2_ADDR: u32 = 0xb00;
pub static LOADER2_SEC_CNT: u32 = 4;

pub static KERNEL_LBA:u32 = LOADER2_LBA + LOADER2_SEC_CNT;
pub static KERENL_ADDR:u32 = 0xc0001500;
pub static KERNEL_SEC_CNT: u32 = 200;

/**
 * 自定义开始的中断号
 */
pub static INTERRUPT_NO_START: u8 = 0x20;