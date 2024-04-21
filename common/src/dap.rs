use core::arch::asm;


/**
 * disk packet address
 * 具体的格式可以看下面的文档
 * <https://en.wikipedia.org/wiki/INT_13H#INT_13h_AH=42h:_Extended_Read_Sectors_From_Drive>
 */
#[repr(C, packed)]
#[allow(dead_code)]
pub struct DiskPacketAddress {
    /**
     * packet的大小。
     * 固定值为16个字节。
     * 就是当前结构体的大小
     */
    size: u8,
    /**
     * 固定为0
     */
    zero: u8,
    /**
     * 读取删除的数量（一些BISO限制最大127字节）
     */
    number_of_sec: u16,
    /**
     * 把数据加载到内存的偏移量
     */
    mem_offset:u16,
    /**
     * 把数据加载到内存的段基址
     */
    mem_segment:u16,
    /**
     * 要读取的硬盘LBA地址
     */
    lba: u64
} 


impl DiskPacketAddress {
    pub fn new(lba: u64, number_of_sec: u16, mem_segment: u16, mem_offset: u16) -> Self{
        return Self {
            size: 0x10,
            zero: 0,
            number_of_sec,
            mem_offset,
            mem_segment,
            lba
        }
    }

    /**
     * 开始执行加载
     */
    pub unsafe fn do_load(&self) {
        // dap的结构的地址
        let address = self as *const Self as u16;
        unsafe{
            asm!(
                "mov {0:x}, si", // 备份si寄存器
                "mov si, {1:x}", // dap的地址
                "mov ah, 0x42", // 固定值0x42
                "mov dl, 0x80", // 固定值0x80，第一个硬盘
                "int 0x13",
                "mov si, {0:x}",
                out(reg) _,
                in(reg) address
            );
        }
    }
}