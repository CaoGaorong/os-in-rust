use core::arch::asm;

/**
 * 禁用中断
 */
pub fn disable_interrupt() {
    unsafe { asm!("cli") };
}
/**
 * 启用中断
 */
pub fn enable_interrupt() {
    unsafe { asm!("sti") };
}

