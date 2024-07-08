
use core::{arch::asm, ptr::{self, addr_of}};

use os_in_rust_common::{idt::{self, InterruptStackFrame, InterruptTypeEnum}, pic, pit, port::Port, sd::SegmentDPL, ASSERT, MY_PANIC};

use crate::{device::{self, ata::{self, ChannelIrqNoEnum}, drive, pio::{self, CommandBlockRegister, StatusRegister}}, interrupt, keyboard::{self, ScanCodeCombinator}, println, scheduler, sys_call::sys_call::{self, HandlerType}, thread};

/**
 * exceptions and codes: <https://wiki.osdev.org/Exceptions>
 */
pub fn init() {
    
    // general protection
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::GeneralProtectionFault, general_protection_handler) }
    // double fault
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::DoubleFault, double_fault_handler) }
    // page fault
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::PageFault, page_fault_handler) }
    // invalid code
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::InvalidOpcode, invalid_opcode_handler) }

    // 初始化时钟中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Timer, timer_handler) }
    // 初始化键盘中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Keyboard, keyboard_handler) }
    
    // 初始化主通道硬盘中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::PrimaryChannel, primary_channel_handler) }
    // 初始化次通道硬盘中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::SecondaryChannel, secondary_channel_handler) }

    // 系统调用中断
    unsafe { idt::IDT.get_mut().set_raw_handler(InterruptTypeEnum::SystemCall, system_call_handler, SegmentDPL::LEVEL3) }
    

    idt::idt_init();

    // 初始化中断控制器
    pic::pic_init();

    pit::pit_init();

}

/**
 * 非法操作码处理
 */
#[cfg(all(not(test), target_arch = "x86"))]
extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    MY_PANIC!("invalid opcode");
}


#[cfg(all(not(target_arch = "x86")))]
fn invalid_opcode_handler(frame: InterruptStackFrame) {
    MY_PANIC!("invalid opcode");
}


/**
 * 保护异常
 */
#[cfg(all(not(test), target_arch = "x86"))]
extern "x86-interrupt" fn general_protection_handler(frame: InterruptStackFrame, error_code: u32) {
    MY_PANIC!("!!!!general protection exception occur, error code:0x{:x}!!!", error_code);
}

#[cfg(all(not(target_arch = "x86")))]
fn general_protection_handler(frame: InterruptStackFrame, error_code: u32) {
    todo!()
}

/**
 * double fault
 */
#[cfg(all(not(test), target_arch = "x86"))]
extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error_code: u32) {
    MY_PANIC!("!!!!!DOUBLE FAULT OCCUR !!!!, error code: 0x{:x}", error_code);
    loop {}
}
#[cfg(all(not(target_arch = "x86")))]
fn double_fault_handler(frame: InterruptStackFrame, error_code: u32) {
    todo!()
}

/**
 * 通用的中断处理程序
 */
#[cfg(all(not(test), target_arch = "x86"))]
extern "x86-interrupt" fn page_fault_handler(frame: InterruptStackFrame, error_code: u32) {
    MY_PANIC!("page fault, code:0x{:x}. eip: 0x{:x}, cs:0x{:x}, eflags:0x{:x}, sp: 0x{:x}, ss:{:x}", error_code, frame.ip as u32, frame.cs as u32, frame.eflags as u32, frame.sp as u32, frame.ss as u32);
}
#[cfg(all(not(target_arch = "x86")))]
fn page_fault_handler(frame: InterruptStackFrame, error_code: u32) {
    MY_PANIC!("page fault, code:0x{:x}. {:?}", error_code, frame);
}


fn alert(error_msg: &str) {
    let vga_buffer = 0xC00b8000 as *mut u8;
    for (i, &e) in error_msg.as_bytes().iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = e;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}




#[cfg(all(not(test), target_arch = "x86"))]
extern "x86-interrupt" fn keyboard_handler(frame: InterruptStackFrame) {
    pic::send_end_of_interrupt();
    // 接收键盘扫描码
    let scan_code = Port::<u8>::new(0x60).read();

    // 对扫描码进行处理
    keyboard::scan_code_handler(scan_code);
}

#[cfg(all(not(target_arch = "x86")))]
fn keyboard_handler(frame: InterruptStackFrame) {
    todo!()
}

#[cfg(all(not(test), target_arch = "x86"))]
pub extern "x86-interrupt" fn timer_handler(frame: InterruptStackFrame) {
    // 进入中断
    stash_intr_stack();

    pic::send_end_of_interrupt();

    // 检查任务的调度。时间片耗尽则调度
    scheduler::check_task_schedule();

    // 中断退出
    pop_intr_stack();
}

#[cfg(all(not(target_arch = "x86")))]
fn timer_handler(frame: InterruptStackFrame) {
    todo!()
}


/**
 * ATA primary channel 的中断
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub extern "x86-interrupt" fn primary_channel_handler(frame: InterruptStackFrame) {
    pic::send_end_of_interrupt();
    // 确保只有内核线程才能收到硬盘中断
    // 否则，当内核线程发起硬盘操作，然后切换到了用户进程，然后收到硬盘中断
    let cur_task = &thread::current_thread().task_struct;
    ASSERT!(cur_task.pgdir == ptr::null_mut());

    let channel_idx = 0;
    let primary_channel = device::get_ata_channel(&channel_idx);
    ASSERT!(primary_channel.is_some());
    let primary_channel = primary_channel.as_mut();
    ASSERT!(primary_channel.is_some());
    let primary_channel = primary_channel.unwrap();
    ASSERT!(primary_channel.irq_no == ChannelIrqNoEnum::Primary as u8);
    // 该通道就绪了
    primary_channel.channel_ready();
}

#[cfg(all(not(target_arch = "x86")))]
fn primary_channel_handler(frame: InterruptStackFrame) {
    todo!()
}

/**
 * ATA secondary channel 的中断
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub extern "x86-interrupt" fn secondary_channel_handler(frame: InterruptStackFrame) {
    pic::send_end_of_interrupt();
    let channel_idx = 1;
    let secondary_channel = device::get_ata_channel(&channel_idx);
    ASSERT!(secondary_channel.is_some());
    let secondary_channel = secondary_channel.as_mut().unwrap(); 
    ASSERT!(secondary_channel.irq_no == ChannelIrqNoEnum::Secondary as u8);
    // 该通道就绪了
    secondary_channel.channel_ready();
}

#[cfg(all(not(target_arch = "x86")))]
fn secondary_channel_handler(frame: InterruptStackFrame) {
    todo!()
}

/**
 * 系统调用；中断处理程序
 * 由于实现系统调用，需要在用户程序和中断处理程序「之间」传递参数（通过寄存器传递），因此我们不能直接保存和恢复上下文，反而要修改上下文。
 * 所以我们必须要知道中断发生到退出具体栈中的上下文数据的变化，因此 **不能使用任何调用约定**（因为使用调用约定我们就不知道栈内的数据情况了）
 */
#[naked]
#[cfg(all(not(test), target_arch = "x86"))]
fn system_call_handler() {
    unsafe {
        asm!(
            // 保存上下文
            "push ds",
            "push es",
            "push fs",
            "push gs",
            "pushad",

            // 传递参数
            "push edx",
            "push ecx",
            "push ebx",
            "push eax",

            // 调用 系统调用分发器，不同的中断号，分发给不同的系统函数
            "call system_call_dispatcher",

            // 回收参数的栈空间
            "add esp, 16",

            // 把栈中eax的地方，修改值。到时候直接恢复到eax寄存器
            "mov [esp + 7*4], eax",

            // 恢复栈的上下文
            "jmp intr_exit",
            options(noreturn),
        )
    }
}

#[cfg(all(not(target_arch = "x86")))]
fn system_call_handler() {
    todo!()
}

/**
 *  系统调用分发器，根据用户程序传递给寄存器的数值，来分发到不同的系统调用函数
 *
 *  **注意，这个函数是给汇编程序调用的，请不要随意修改函数名称**
 */
#[no_mangle]
extern "C" fn system_call_dispatcher(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {

    // println!("eax:{}, ebx:{}, ecx:{}, edx:{}", eax, ebx, ecx, edx);

    // 根据系统调用号，找到系统调用函数
    let handler_opt = sys_call::get_handler(eax);
    if handler_opt.is_none() {
        MY_PANIC!("system call invoke error, no handler found");
    }
    
    let sys_handler = handler_opt.unwrap();

    // 匹配系统调用函数的参数个数，进行调用
    let res = match sys_handler {
        // 无参数的
        HandlerType::NoneParam(func) => {
            func()
        },
        // 1个参数的
        HandlerType::OneParam(func) => {
            func(ebx)
        },
        // 2个参数的
        HandlerType::TwoParams(func) => {
            func(ebx, ecx)
        },
        // 3个参数的
        HandlerType::ThreeParams(func) => {
            func(ebx, ecx, edx)
        },
    };
    // 参数返回，由于是C调用约定，因此该参数放在eax寄存器中
    res
}

/**
 * 保存中断上下文
 */
#[inline(always)]
#[cfg(all(not(test), target_arch = "x86"))]
fn stash_intr_stack() {
    // 保护上下文，主要是保护段寄存器
    unsafe {
        asm!(
            "push ds",
            "push es",
            "push fs",
            "push gs",
            "pushad",
        )
    }
}

#[inline(always)]
#[cfg(all(not(target_arch = "x86")))]
fn stash_intr_stack() {
    todo!()
}


/**
 * 恢复中断上下文
 */
#[inline(always)]
#[no_mangle]
#[cfg(all(not(test), target_arch = "x86"))]
fn pop_intr_stack() {
    unsafe {
        asm!(
            "popad",
            "pop gs",
            "pop fs",
            "pop es",
            "pop ds",
        )
    }
}

#[inline(always)]
#[no_mangle]
#[cfg(all(not(target_arch = "x86")))]
fn pop_intr_stack() {
    todo!()
}

/**
 * 退出中断；恢复手动上下文 + iretd
 * 这个函数给汇编程序调用，不要改名
 */
#[inline(always)]
#[no_mangle]
#[cfg(all(not(test), target_arch = "x86"))]
pub fn intr_exit() {
    // 恢复手动入栈的上下文
    pop_intr_stack();
    // 在使用iret恢复CPU入栈的上下文
    unsafe {
        asm!("iretd")
    }
}

#[inline(always)]
#[no_mangle]
#[cfg(all(not(target_arch = "x86")))]
pub fn intr_exit() {
    todo!()
}




