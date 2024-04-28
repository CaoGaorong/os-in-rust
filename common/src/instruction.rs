use core::arch::asm;

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
    unsafe { asm!("sti", options(nomem, nostack));};
}


