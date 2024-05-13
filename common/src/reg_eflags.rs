use core::arch::asm;

pub enum FlagEnum {
    // 0位
    CarryFlag = 0x0,
    // 1位
    FirstReserved,
    // 2位
    ParityFlag,
    // 3位
    ThirdReserved,
    // 4位
    AuxiliaryFlag,
    // 5位
    FifthReserved,
    // 6位
    ZeroFlag,
    // 7位
    SignFlag,
    // 8位
    TrapFlag,
    // 9位
    InterruptFlag,
    // 10位
    DirectionFlag,
    // 11位
    OverflowFlags,
    // 12位
    InputOutputPrivilegeLevel,
    // 14位
    NestedFlag = 0xe,
    // 15位
    FifteenReserved,
    // 16位
    ResumeFlag,
    // 17位
    VirtualMode,
    // 18位
    AlignmentCheck,
    // 19位
    VirtualInterruptFlag,
    // 20位
    VirtualInterruptPending,
    // 21位
    Identification,
}

/**
 * 查看某个标志是否开启
 */
#[inline]
pub fn is_flag_on(flag: FlagEnum) -> bool {
    let mut eflags: u32;
    unsafe {
        // 保存上下文
        // 把eflags的值放到eax，然后赋值给变量
        asm!(
            "pushf",
            "pop {0:e}",
            out(reg) eflags,
            
            // TODO 奇怪。这里必须使用att风格，如果是intel风格就报错了
            options(att_syntax)
        );
    }
    let flag_idx = flag as u8;
    ((eflags & (1 << flag_idx)) >> flag_idx) == 0x1
}
