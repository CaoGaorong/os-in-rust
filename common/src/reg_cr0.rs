use core::arch::asm;

/**
 * 本文件是对cr0寄存器的直接操作
 * cr0寄存器的结构：<https://wiki.osdev.org/CPU_Registers_x86#CR0>
 */

pub enum CR0 {
    /**
     * Protected mode Enable
     */
    PE = 0,
    /**
     * Monitor co-Processor
     */
    MP = 1,
    /**
     * x87 FPU Emulation
     */
    EM = 2,
    /**
     * Task Switch
     */
    TS = 3,
    /**
     * Extension Type
     */
    ET = 4,
    /**
     * Numberic Error
     */
    NE = 5,
    /**
     * Write Protect
     */
    WP = 16,
    /**
     * Alignment Mask
     */
    AM = 18,
    /**
     * Not Write throught
     */
    NW = 29,
    /**
     * Cache Disable
     */
    CD = 30,
    /**
     * Pageing
     */
    PG = 31,
}

pub fn set_on(reg: CR0) -> u32{
    let mut cr0_before: u32;
    unsafe {
        asm!("mov {:e}, cr0", out(reg) cr0_before, options(nomem, nostack, preserves_flags));
    }
    // 把原本cr0对应这一位设置为1
    let cr0_after = cr0_before | 1 << (reg as u8);
    // 写入到cr0寄存器
    set_cr0(cr0_after);
    // 返回已有的寄存器
    cr0_before
}

fn set_off(reg: CR0) -> u32{
    let mut cr0_before: u32;
    unsafe {
        asm!("mov {:e}, cr0", out(reg) cr0_before, options(nomem, nostack, preserves_flags));
    }
    // 把原本cr0对应这一位设置为0
    let cr0_after = cr0_before & 0 << (reg as u8);
    // 写入到cr0寄存器
    set_cr0(cr0_after);
    // 返回已有的寄存器
    cr0_before
}


fn set_cr0(val: u32) {
    unsafe { asm!("mov cr0, {:e}", in(reg) val, options(nostack, preserves_flags)) };
}

