use core::{arch::asm, marker::PhantomData};

use crate::printk;

pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>
}

impl <T:PortRead + PortWrite> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }
}

impl <T:PortRead> Port<T> {
    pub fn read(&self) -> T {
        T::read_from_port(self.port)
    }
}

impl <T:PortWrite> Port<T> {
    pub fn write(&self, value: T) {
        T::write_to_port(self.port, value)
    }
}


/**
 * 定义写端口的接口
 */
trait PortWrite {
    fn write_to_port(port: u16, value: Self);
}

/**
 * 定义读数据的接口
 */
trait PortRead {
    fn read_from_port(port: u16) -> Self;
}

impl PortRead for u8 {
    fn read_from_port(port: u16) -> Self {
        let value: u8;
        unsafe {
            asm!("in al, dx", in("dx") port, out("al") value);
        }
        value
    }
}
impl PortRead for u16 {
    fn read_from_port(port: u16) -> Self {
        let value: u16;
        unsafe {
            asm!("in ax, dx", in("dx") port, out("ax") value);
        }
        value
    }
}

impl PortWrite for u8 {
    fn write_to_port(port: u16, value: Self) {
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
        }
    }
}
impl PortWrite for u16 {
    #[cfg(not(test))]
    fn write_to_port(port: u16, value: Self) {
        unsafe {
            asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
        }
    }
}


/**
 * 从port中连续读取word_cnt个字的数据到buf_addr地址处的内存中
 */
#[cfg(not(test))]
#[no_mangle]
#[inline(never)]
pub fn read_words(port: u16, word_cnt: u32, buf_addr: u32) {
    /*
     * 利用insw指令。insw指令需要两个参数：
     * - dx: 要读取数据的端口号
     * - es:di: 将要把数据写入的内存地址
     * 
     * 而rep指令是循环，需要ecx作为循环次数
     */
    unsafe {
        asm!(
            "cld",
            "rep insw",
            in("dx") port,
            in("edi") buf_addr,
            in("ecx") word_cnt
        );
    }
}
#[cfg(not(test))]
#[no_mangle]
#[inline(never)]
pub fn write_words(port: u16, buf_addr: u32, word_cnt: u32) {
    unsafe {
        asm!(
            "mov esi, {0:e}",
            "cld",
            "rep outsw",
            in(reg) buf_addr,
            in("dx") port,
            in("ecx") word_cnt
        );
    }
}