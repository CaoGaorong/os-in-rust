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
pub fn disable_interrupt() -> InterruptStatus {
    if is_intr_on() {
        unsafe { asm!("cli") };
        return InterruptStatus::On;
    }
    return InterruptStatus::Off;
}
/**
 * 启用中断
 */
#[inline]
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


/**
 * 执行hlt指令
 */
pub fn halt() {
    unsafe {
        asm!(
            "sti",
            "hlt"
        )
    }
}
