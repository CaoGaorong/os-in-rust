use core::arch::asm;

use crate::reg_eflags;

/**
 * 禁用中断
 */
#[inline]
pub fn disable_interrupt() {
    unsafe { asm!("cli") };
}
/**
 * 启用中断
 */
#[inline]
pub fn enable_interrupt() {
    unsafe { asm!("sti");};
}

/**
 * 中断有没有开
 * TODO 我发现，调用这个方法必须初始化了中断描述符等信息才可以，否则会报错。很奇怪的报错。pop eflags的值的时候会报错
 */
pub fn is_intr_on() -> bool {
    // 查看eflags寄存器
    reg_eflags::is_flag_on(reg_eflags::FlagEnum::InterruptFlag)
}


