use core::arch::asm;

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
    Code0Selector = 0b1_0_00,
    /**
     * 0级数据段选择子
     * Index = 2
     * TI = 0
     * RPL = 0
     */
    Data0Selector = 0b10_0_00,
}


/**
 * 加载数据段选择子到段寄存器中
 */
pub fn load_data_selector() {
    let data_selector = SegmentSelector::Data0Selector as u16;
    // load GDT
    unsafe {
        asm!("mov ds, {0:x}", "mov ss, {0:x}", "mov es, {0:x}", in(reg) data_selector);
    }
}
