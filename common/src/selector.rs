use core::arch::asm;

use crate::gdt::DescriptorType;

/**
 * 段选择子的结构
 * <https://wiki.osdev.org/Segment_Selector>
 *
 *  15                                   3  2  1      0
 * +--------------------------------------|---|--------+
 * |                 Index                |TI |  RPL   |
 * +--------------------------------------|---|--------+
 *
 */
pub enum SegmentSelector {
    /**
     * 0级代码段选择子
     * Index = 1
     * TI = 0
     * RPL = 0
     */
    Code0Selector = ((DescriptorType::Code as u8) << 3) as isize |  0b0_00,
    /**
     * 0级数据段选择子
     * Index = 2
     * TI = 0
     * RPL = 0
     */
    Data0Selector = ((DescriptorType::Data as u8) << 3) as isize |  0b0_00,

    /**
     * 显存段选择子
     * Index = 3
     * TI = 0
     * RPL = 0
     */
    VideoSelector = ((DescriptorType::Video as u8) << 3) as isize |  0b0_00,

    /**
     * TSS段选择子
     * Index = 4
     * TI = 0
     * RPL = 0
     */
    TssSelector = ((DescriptorType::Tss as u8) << 3) as isize |  0b0_00,

    /**
     * 用户代码段选择子
     * Index = 5
     * TI = 0
     * RPL = 3
     */
    UserCodeSelector = ((DescriptorType::UserCode as u8) << 3) as isize |  0b0_11,

    /**
     * 用户数据段选择子
     * Index = 6
     * TI = 0
     * RPL = 3
     */
    UserDataSelector = ((DescriptorType::UserData as u8) << 3) as isize |  0b0_11,
}


/**
 * 加载数据段选择子到段寄存器中
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub fn load_data_selector() {
    let data_selector = SegmentSelector::Data0Selector as u16;
    // load GDT
    unsafe {
        asm!("mov ds, {0:x}", "mov ss, {0:x}", "mov es, {0:x}", in(reg) data_selector);
    }
}

#[cfg(all(not(target_arch = "x86")))]
pub fn load_data_selector() {
    todo!()
}
