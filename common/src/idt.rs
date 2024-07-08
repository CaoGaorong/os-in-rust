use core::{arch::asm, mem::size_of};

use crate::{constants, gdt::DescriptorType, racy_cell::RacyCell, sd::SegmentDPL, selector::SegmentSelector, utils};


/**
 * 中断描述符表相关的信息，构建中断描述符。
 * 中断号0x00 - 0x1F是固定保留给CPU的，可以看<https://en.wikipedia.org/wiki/Interrupt_descriptor_table#Processor-generated_exceptions>
 * 那么我们的自定义中断，就从0x20开始
 */

/**
 * 构建一个空的中断描述符表
 */
#[no_mangle]
pub static IDT: RacyCell<InterruptDescriptorTable> = RacyCell::new(InterruptDescriptorTable::empty());


/**
 * 初始化中断描述符表
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub fn idt_init() {
    unsafe { IDT.get_mut().load() };
}
#[cfg(not(target_arch = "x86"))]
pub fn idt_init() {
    todo!()
}


const IDT_LENGTH:usize  = 0x81;

/**
 * 中断的类型，包括两部分：
 * - CPU保留的异常：可以看<https://en.wikipedia.org/wiki/Interrupt_descriptor_table#Processor-generated_exceptions>
 * - 8259A的异常，自定义从0x20编号开始。关于8259A可以看：<https://os.phil-opp.com/hardware-interrupts/#the-8259-pic>
 */
pub enum InterruptTypeEnum {
    // 下面15个是CPU保留的异常，固定的中断号
    DivideByZero = 0x00,
    SingleStepDebug = 0x01,
    NonMaskInterrupt = 0x02,
    BreakPoint = 0x03,
    Overflow = 0x04,
    BoundRangeExceed = 0x05,
    InvalidOpcode = 0x06,
    CoprocessorNotAvailable = 0x07,
    DoubleFault = 0x08,
    CoprocessorSegmentOverrun = 0x09,
    InvalidTaskStateSegment = 0x0A,
    SegmentNotPresent = 0x0B,
    StackSegmentFault = 0x0C,
    GeneralProtectionFault = 0x0D,
    PageFault = 0x0E,
    Reserved = 0x0F,
    X87FloatingPointException = 0x10,
    AlignmentCheck = 0x11,
    MachineCheck = 0x12,
    SIMDFloatingPointException = 0x13,
    VirtualizationException = 0x14,
    ControlProtectionException = 0x15,


    // 下面的异常，是自定义的，来自中断控制器
    // 时钟中断
    Timer = 0x20,
    // 键盘中断
    Keyboard = 0x21,

    // 硬盘ATA的主通道，中断号
    PrimaryChannel = (constants::INTERRUPT_NO_START + 14) as isize,
    // 硬盘ATA的次通道，中断号
    SecondaryChannel = (constants::INTERRUPT_NO_START + 15) as isize,


    SystemCall = 0x80,
}



/**
 * 没有错误码的中断处理函数定义
 */
// #[cfg(feature = "abi_x86_interrupt")]
#[cfg(all(not(test), target_arch = "x86"))]
pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);

#[cfg(all(not(target_arch = "x86")))]
pub type HandlerFunc = fn(InterruptStackFrame);


/**
 * 有错误码的中断处理函数的定义
 */
// #[cfg(feature = "abi_x86_interrupt")]
#[cfg(all(not(test), target_arch = "x86"))]
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u32);
#[cfg(all(not(target_arch = "x86")))
]
pub type HandlerFuncWithErrCode = fn(InterruptStackFrame, error_code: u32);


/**
 * 中断发生时的栈
 *  当中断发生，CPU前往IDT寻找到中断描述符中的中断处理程序，然后调用。调用之前需要做一些保存上下文的动作（CPU完成）
 *  保存上下文如下：
 *      - 压入旧栈的SS
 *      - 压入旧栈的SP
 *      - 压入EFLAGS寄存器
 *      - 压入当前的CS寄存器
 *      - 压入当前的IP寄存器
 *      - 如果有错误码，还要压入错误码
 * 栈的地址是从高到低，所以定义一个结构，取出栈的数据，跟压栈顺序反着来
 * InterruptStackFrame的结构见：<https://os.phil-opp.com/handling-exceptions/#the-exception-stack-frame>
 *
 * > 注意，这里不包括「中断处理程序」自身保存的寄存器上下文的值，仅仅是CPU保存上下文的值
 */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct InterruptStackFrame {
    pub ip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub sp: u32,
    pub ss: u32,
}


/**
 * 中断描述符的结构
 * 8个字节（64位）
 * <https://os.phil-opp.com/cpu-exceptions/#the-interrupt-descriptor-table>
 */
#[derive(Clone, Copy)]
#[repr(C, packed)]
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
    pub const fn empty() -> Self {
        Self {
            code_addr_low: 0,
            code_selector: 0,
            reserved_zero: 0,
            attr: 0,
            code_addr_high: 0
        }
    }
    pub fn new(code_selector: SegmentSelector, code_addr: u32, present: bool, dpl: SegmentDPL) -> Self {
        Self {
            code_selector: code_selector as u16,
            code_addr_low: code_addr as u16,
            code_addr_high: (code_addr >> 16) as u16,
            reserved_zero: 0,
            attr: 
                // P位
                (utils::bool_to_int(present) << 7) as u8| 
                // DPL位
                (dpl as u8) << 5 | 
                // S + TYPE位，固定值
                0b01110 as u8
        }
    }
}

/**
 * 中断描述符表
 * 这里中断描述符个数是0x81。因为我们系统调用中断号是0x80
 */
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct InterruptDescriptorTable {
    data: [InterruptDescriptor; IDT_LENGTH]
}

impl InterruptDescriptorTable {
    
    /**
     * 构建空的中断描述符表
     */
    const fn empty() -> Self {
        Self {
            data: [InterruptDescriptor::empty(); IDT_LENGTH]
        }
    }

    /**
     * 给中断描述符表的某一个中断描述符赋值
     */
    #[inline]
    pub fn set_interrupt(&'static mut self, interrupt_type: InterruptTypeEnum, descriptor: InterruptDescriptor) {
        self.data[interrupt_type as usize] = descriptor;
    }

    /**
     * 设置中断处理函数
     *   interrupt_type: 中断的类型
     *   handler: 中断处理函数
     */
    pub fn set_handler(&'static mut self, interrupt_type: InterruptTypeEnum, handler: HandlerFunc) {
        let id = InterruptDescriptor::new(SegmentSelector::Code0Selector, handler as *const() as u32, true, SegmentDPL::LEVEL0);
        self.set_interrupt(interrupt_type, id);
    }

    pub fn set_raw_handler(&'static mut self, interrupt_type: InterruptTypeEnum, handler: fn(), dpl: SegmentDPL) {
        let id = InterruptDescriptor::new(SegmentSelector::Code0Selector, handler as *const() as u32, true, dpl);
        self.set_interrupt(interrupt_type, id);
    }

    /**
     * 设置有错误码的中断处理函数
     *   interrupt_type: 中断的类型
     *   handler: 中断处理函数
     */
    pub fn set_error_code_handler(&'static mut self, interrupt_type: InterruptTypeEnum, handler: HandlerFuncWithErrCode) {
        let id = InterruptDescriptor::new(SegmentSelector::Code0Selector, handler as *const () as u32, true, SegmentDPL::LEVEL0);
        self.set_interrupt(interrupt_type, id);
    }

    #[cfg(all(not(test), target_arch = "x86"))]
    pub fn load(&'static self) {
        let idtr = IDTR::new(self as *const InterruptDescriptorTable);
        // 加载到GDTR寄存器
        unsafe {
            asm!("cli", "lidt [{}]", in(reg) &idtr, options(readonly, nostack, preserves_flags));
        }
    }
    #[cfg(all(not(target_arch = "x86")))]
    pub fn load(&'static self) {
        todo!()
    }

}

/**
 * 中断描述符表寄存器结构
 * 
 */
#[repr(C,packed)]
pub struct IDTR {
    /**
     * 中断描述符表的长度。单位字节
     */
    idt_limit: u16,
    /**
     * 中断描述符表的起始地址
     */
    idt_addr: u32
}

impl IDTR {
    fn new(idt: *const InterruptDescriptorTable) -> Self {
        Self {
            idt_limit: (size_of::<InterruptDescriptorTable>() - 1) as u16,
            idt_addr: idt as u32
        }
    }
}