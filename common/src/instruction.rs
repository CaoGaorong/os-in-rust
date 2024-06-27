use core::arch::asm;

use crate::{printkln, reg_eflags};

#[derive(PartialEq, Debug)]
pub enum InterruptStatus {
    Off,
    On
}
/**
 * 禁用中断
 */
#[inline]
#[cfg(all(not(test), target_arch = "x86"))]
pub fn disable_interrupt() -> InterruptStatus {
    if is_intr_on() {
        unsafe { asm!("cli") };
        return InterruptStatus::On;
    }
    return InterruptStatus::Off;
}

#[inline]
#[cfg(any(test, not(target_arch = "x86")))]
pub fn disable_interrupt() -> InterruptStatus {
    todo!()
}

/**
 * 启用中断
 */
#[inline]
#[cfg(any(test, not(target_arch = "x86")))]
pub fn enable_interrupt() -> InterruptStatus {
    todo!()
}

#[inline]
#[cfg(all(not(test), target_arch = "x86"))]
pub fn enable_interrupt() -> InterruptStatus {
    if is_intr_on() {
        return InterruptStatus::On;
    }
    unsafe { asm!("sti");};
    return InterruptStatus::Off;
}

pub fn set_interrupt(status: InterruptStatus) {
    if InterruptStatus::On == status {
        enable_interrupt();
    } else {
        disable_interrupt();
    }
}

/**
 * 中断有没有开
 * TODO 我发现，调用这个方法必须初始化了中断描述符等信息才可以，否则会报错。很奇怪的报错。pop eflags的值的时候会报错
 */
#[inline]
pub fn is_intr_on() -> bool {
    // 查看eflags寄存器
    reg_eflags::is_flag_on(reg_eflags::FlagEnum::InterruptFlag)
}

#[cfg(all(not(test), target_arch = "x86"))]
pub fn load_esp() -> u32 {
    let cur_esp: u32;
    unsafe {
        asm!(
            "mov {0:e}, esp",
            out(reg) cur_esp,
        );
    }
    cur_esp
}

#[inline]
#[cfg(any(test, not(target_arch = "x86")))]
pub fn load_esp() -> u32 {
    todo!()
}


/**
 * 执行hlt指令
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub fn halt() {
    unsafe {
        asm!(
            "sti",
            "hlt"
        )
    }
}
#[cfg(all(not(target_arch = "x86")))]
pub fn halt() {
    todo!()
}


/**
 * 清除页表缓存
 * <https://wiki.osdev.org/TLB>
 */
pub fn invalidate_page(vaddr: usize) {
    unsafe {
        asm!(
            "invlpg [{:e}]",
            in(reg) vaddr,
            options(nostack, preserves_flags)
        )
    }
}