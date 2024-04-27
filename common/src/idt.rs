use crate::{sd::SegmentDPL, utils};

/**
 * 中断描述符表相关的信息
 */

/**
 * 中断描述符的结构
 * 8个字节（64位）
 */
#[repr(C)]
pub struct InterruptDescriptor {
    /**
     * 代码段的地址，低16位
     */
    code_addr_low: u16,
    /**
     * 代码段所在内存段的选择子
     */
    code_selector: u16,
    /**
     * 保留位。都为0
     */
    reserved_zero: u8,
    /**
     * 属性
     * P + DPL + S + TYPE
     */
    attr: u8,
    /**
     * 代码段的地址，高16位
     */
    code_addr_high: u16
}
impl InterruptDescriptor {
    pub fn new(code_selector: u16, code_addr: u32, present: bool, dpl: SegmentDPL) -> Self {
        Self {
            code_selector,
            code_addr_low: code_addr as u16,
            code_addr_high: (code_addr >> 16) as u16,
            reserved_zero: 0,
            attr: 
                // P位
                (utils::bool_to_int(present) << 7) as u8| 
                // DPL位
                (dpl as u8) << 5 | 
                // S + TYPE位，固定值
                0b11110 as u8
        }
    }
}

/**
 * 中断描述符表
 * 这里中断描述符个数是0x81。因为我们系统调用中断号是0x80
 */
#[repr(transparent)]
pub struct InterruptDescriptorTable {
    data: [InterruptDescriptor; 0x81]
}

impl InterruptDescriptorTable {
    
}
/**
 * 中断描述符表寄存器结构
 */
pub struct IDTR {

}