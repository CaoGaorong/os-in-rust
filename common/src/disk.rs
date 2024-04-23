use core::{arch::asm, mem, num};

use crate::utils;

/**
 * 主通道的命令寄存器的可选枚举
 */
enum PrimaryCommandRegister {
    Data = 0x1F0,
    ErrorFeature = 0x1F1,
    SectorCount = 0x1F2,
    LBALow = 0x1F3,
    LBAMid = 0x1F4,
    LBAHigh = 0x1F5,
    Device = 0x1F6,
    StatusCommand = 0x1F7,
}

#[repr(transparent)]
struct DeviceRegister {
    data: u8
}
impl DeviceRegister {
    fn new(lba: u8, primary: bool, is_lba: bool) -> Self {
        Self {
            data: 
                (lba & 0b1111)  |
                // 0代表主盘，1代表从盘
                ((if primary {0} else {1}) << 4) as u8|
                1 << 5 |
                (utils::bool_to_int(is_lba) << 6) as u8 |
                1 << 7
        }
    }
    fn get_data(&self) -> u8 {
        self.data
    }
}

/**
 * 命令寄存器的值
 */
enum CommandRegister {
    // 识别硬盘命令
    Identify = 0xEC,
    // 读取硬盘命令
    Read = 0x20,
    // 写入硬盘命令
    Write = 0x30,
}


/**
 * <https://doc.rust-lang.org/reference/inline-assembly.html#template-modifiers>
 */

/**
 * 读取硬盘
 * lba: 读取的LBA地址。最高27bit
 * num_sec: 要读取的扇区数量；最多255。0表示读取256个扇区
 * mem_addr: 要加载到的内存地址
 */
pub fn read_disk(lba: u32, num_sec: u8, mem_addr: u32) {
    // 先写入读磁盘的命令
    read_disk_command(lba, num_sec);
    // 查看磁盘准备好了没有
    while !disk_ready() {}
    // 从磁盘中读取数据到指定内存空间
    disk_read(mem_addr, num_sec as u32 * 512)
}

fn read_disk_command(lba: u32, num_sec: u8) {
    let num = num_sec as u16;
    let lba_low = lba as u16;
    let lba_mid = (lba >> 8) as u16;
    let lba_high = (lba >> 16) as u16;
    // LBA的最高4位
    let lba_max  = ((lba >> 24) & 0b1111) as u8;
    // 构建好device寄存器的值
    let device_val = DeviceRegister::new(lba_max, true, true).get_data() as u16;
    let command_val = CommandRegister::Read as u16;
    
    // 第一步，写入sector count；0x1F2是写入selector count的端口号
    unsafe {asm!("mov dx, 0x1F2", "mov ax, {0:x}", "out dx, al", in(reg) num, options(nomem, nostack, preserves_flags))}
    
    // 第二步，写入LBA Low; 
    unsafe {asm!("mov dx, 0x1F3", "mov ax, {0:x}", "out dx, al", in(reg) lba_low, options(nomem, nostack, preserves_flags))}
    
    // 第三步，写入LBA Mid
    unsafe {asm!("mov dx, 0x1F4", "mov ax, {0:x}", "out dx, al", in(reg) lba_mid, options(nomem, nostack, preserves_flags))}
    
    // 第四步，写入LBA High
    unsafe {asm!("mov dx, 0x1F5", "mov ax, {0:x}", "out dx, al", in(reg) lba_high, options(nomem, nostack, preserves_flags))}
    
    // 第五步，写入device寄存器
    unsafe {asm!("mov dx, 0x1F6", "mov ax, {0:x}", "out dx, al", in(reg) device_val, options(nomem, nostack, preserves_flags))}

    // 第六步，发起读命令 command寄存器
    unsafe {asm!("mov dx, 0x1F7", "mov ax, {0:x}", "out dx, al", in(reg) command_val, options(nomem, nostack, preserves_flags))}
    
}

fn disk_ready() -> bool {
    let mut status:u16 = 0;
    unsafe { asm!("mov dx, 0x1F7", "in al, dx", "mov ax, {0:x}", out(reg) status, options(nomem, nostack, preserves_flags))}
    // status寄存器的结构。当且仅当第4位是1，第8位是0的时候，才可以
    return (status & 0b10001000) == 0b1000;
}
/**
 * 从磁盘中读取数据
 * addr_to_save: 数据读取后存储到的内存地址
 * num_bytes: 要读取的字节数
 */
fn disk_read(addr_to_save: u32, num_bytes: u32) {
    // 要读取多少次。一共numb_sec * 512个字节，每次读2字节
    let loop_count = num_bytes / 2;
    // let mem_addr = addr_to_save as u16 as *mut u16;
    let mut addr = addr_to_save;

    let mut data:u16 = 0;
    for i in 0 .. loop_count {
        unsafe {
            asm!(
                "push eax",
                "mov dx, 0x1F0", // data register port
                "in ax, dx", // 把dx指定的端口的数据读取出来
                "mov {0:x}, ax",
                "pop eax",
                out(reg) data
                // options(nomem, nostack, preserves_flags)
            )
        }
        unsafe { *(addr as *mut u16) = data };
        addr += 2;
        // 把从dx读取出来的数据，复制到内存地址中
        // unsafe { *mem_addr.offset(i as isize * 2) =  data };
    }
    
}