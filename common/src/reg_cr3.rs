use core::arch::asm;

/**
 * 寄存器结构：<https://wiki.osdev.org/CPU_Registers_x86>
 */

use crate::{paging::PageTable, printkln};

 /**
  * cr3寄存器的长度32位
  */
#[repr(transparent)]
pub struct CR3 {
    pub data: u32
}


impl CR3 {
    /**
     * 构建一个CR3寄存器的数据
     * addr: 页表的起始地址
     */
    pub fn new(addr: *const PageTable) -> Self {
        let addr_val = addr as *const u32 as u32;
        // 构建cr3。CR3寄存器数据的高20位是GDT地址的高20位，低20位是0
        Self {
            data: (addr_val >> 12) << 12
        }
    }
    pub fn load_cr3(&self) {
        unsafe { asm!("mov cr3, {:e}", in(reg) self.data) };
    }

}
