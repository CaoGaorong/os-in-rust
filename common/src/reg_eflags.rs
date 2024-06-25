use core::arch::asm;

/**
 * eflags
 * <https://en.wikipedia.org/wiki/FLAGS_register>
 */
pub enum FlagEnum {
    // 0位
    CarryFlag = 0x0,
    // 1位。保留，永远始终为1
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

#[repr(C, packed)]
pub struct EFlags {
    data: u32
}

impl EFlags {
    pub fn empty() -> Self {
        Self {
            data: 0
        }
    }
    /**
     * 读取寄存器的Eflags数据
     */
    #[cfg(not(test))]
    pub fn load() -> Self {
        let mut eflags: u32;
        unsafe {
            // 保存上下文
            // 把eflags的值放到eax，然后赋值给变量
            asm!(
                "pushf",
                "pop {0:e}",
                out(reg) eflags,
                options(att_syntax)
            );
        }
        Self {
            data: eflags
        }
    }
    /**
     * 把eflags寄存器某一位打开
     */
    pub fn set_on(&mut self, flag: FlagEnum) {
        self.data = self.data | (1 << flag as u8);
    }

    /**
     * 把eflags寄存器某一位关闭
     */
    pub fn set_off(&mut self, flag: FlagEnum) {
        self.data = self.data & !(1 << flag as u8);
    }

    pub fn get_data(&self) -> u32 {
        self.data
    }

    /**
     * 把数据写入到寄存器中
     */
    #[cfg(not(test))]
    pub fn store(&self) {
        unsafe {
            asm!(
                // 把数据压入栈中
                "push {:e}",
                // 把栈的数据弹出到eflags
                "popf",
                in(reg) self.data,
                options(att_syntax)
            )
        }
    }
}
/**
 * 查看某个标志是否开启
 */
#[inline]
pub fn is_flag_on(flag: FlagEnum) -> bool {
    // 读取eflags
    let eflags = EFlags::load().data;
    let flag_idx = flag as u8;
    ((eflags & (1 << flag_idx)) >> flag_idx) == 0x1
}
