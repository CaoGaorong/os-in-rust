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
     * 读取删除的数量（一些BISO限制最大127个扇区）
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
    #[cfg(all(not(test), target_arch = "x86"))]
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
    #[cfg(all(not(target_arch = "x86")))]
    pub unsafe fn do_load(&self) {
        todo!()
    }
}

/**
 * 加载磁盘
 * lba: 磁盘的LBA地址
 * num_sec: 加载的扇区数量
 * mem_addr: 要加载到的内存地址
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub fn load_disk(lba: u64, num_sec: u16, mem_addr: u32) {
    // 取Loader加载到内存地址的高16位
    let addr_high16 = (mem_addr >> 16) as u16;
    // 取loader加载到的内存地址的低16位
    let addr_low16 = mem_addr as u16;
    // 构建Disk Packet Address
    let dap_structre = DiskPacketAddress::new(
        lba,
        num_sec,
        addr_high16,
        addr_low16,
    );

    // 开始执行，把硬盘的数据加载到内存
    unsafe {
        dap_structre.do_load();
    }
}

#[cfg(all(not(target_arch = "x86")))]
pub fn load_disk(lba: u64, num_sec: u16, mem_addr: u32) {
    todo!()
}