
use core::{arch::asm, ptr::addr_of};

use os_in_rust_common::{constants, idt::{self, HandlerFunc, InterruptStackFrame, InterruptTypeEnum}, instruction, pic, pit, port::Port, print, println, ASSERT};

use crate::{interrupt, keyboard::{self, ScanCodeCombinator}, scheduler, sys_call::{self, HandlerType}, thread};

pub fn init() {
    
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::GeneralProtectionFault, general_protection_handler) }
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::DoubleFault, general_protection_handler) }
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::PageFault, general_protection_handler) }

    // 初始化时钟中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Timer, timer_handler) }
    // 初始化键盘中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Keyboard, keyboard_handler) }
    
    // 系统调用中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::SystemCall, system_call_handler) }
    

    idt::idt_init();

    // 初始化中断控制器
    pic::pic_init();

    pit::pit_init();

}



/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn general_handler(frame: InterruptStackFrame) {
    print!(".");
    pic::send_end_of_interrupt();
}

/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn system_call_handler(frame: InterruptStackFrame) {
    // 保存中断上下文
    stash_intr_stack();

    // 取出系统调用中的所有参数
    let sys_no: u32;
    let param1: u32;
    let param2: u32;
    let param3: u32;
    unsafe {
        asm!(
            "mov eax, eax",
            "mov ebx, ebx",
            "mov ecx, ecx",
            "mov edx, edx",
            out("eax") sys_no,
            out("ebx") param1,
            out("ecx") param2,
            out("edx") param3,
        )
    }

    // println!("no :0x{:x}, p1: 0x{:x}, p2: 0x{:x}, p3: 0x{:x}", sys_no, param1, param2, param3);

    let sys_handler = sys_call::get_handler(sys_no);
    let res = match sys_handler {
        HandlerType::NoneParam(func) => {
            func()
        },
        HandlerType::OneParam(func) => {
            func(param1)
        },
        HandlerType::TwoParams(func) => {
            func(param1, param2)
        },
        HandlerType::ThreeParams(func) => {
            func(param1, param2, param3)
        },
    };
    
    let esp: u32;
    unsafe {
        asm!(
            "mov {:e}, esp",
            out(reg) esp
        )
    }
    let intr_stack = unsafe { &mut *(esp as *mut thread::InterruptStack) };
    intr_stack.eax = res;
    
    // 恢复中断上下文
    pop_intr_stack();
}


/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn general_handler_with_error_code(frame: InterruptStackFrame, error_code: u32) {
    pic::send_end_of_interrupt();
    println!("!!!!general error code exception occur!!!");
    loop {}
}


extern "x86-interrupt" fn keyboard_handler(frame: InterruptStackFrame) {
    pic::send_end_of_interrupt();
    // 接收键盘扫描码
    let scan_code = Port::<u8>::new(0x60).read();

    // 对扫描码进行处理
    keyboard::scan_code_handler(scan_code);
}


/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn general_protection_handler(frame: InterruptStackFrame, error_code: u32) {
    pic::send_end_of_interrupt();
    println!("!!!!general protection exception occur!!!");
    loop {}
}

pub extern "x86-interrupt" fn timer_handler(frame: InterruptStackFrame) {

    // 进入中断
    stash_intr_stack();

    pic::send_end_of_interrupt();
    let current_thread = thread::current_thread();
    let task_name = current_thread.task_struct.name;
    // 确保栈没有溢出
    ASSERT!(current_thread.task_struct.stack_magic == constants::TASK_STRUCT_STACK_MAGIC);
    let task_struct = &mut current_thread.task_struct;

    // 该进程运行的tick数+1
    task_struct.elapsed_ticks += 1;

    // 如果剩余的时间片还有，那就减少
    if task_struct.left_ticks > 0 {
        task_struct.left_ticks -= 1;
    } else {
        // 否则就切换其他线程
        scheduler::schedule();
        // println!("schedule finished");
    }

    // 中断退出
    pop_intr_stack();
    
}


/**
 * 保存中断上下文
 */
#[inline(always)]
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

/**
 * 恢复中断上下文
 */
#[inline(always)]
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

/**
 * 退出中断；恢复手动上下文 + iretd
 */
#[inline(always)]
pub fn intr_exit() {
    // 恢复手动入栈的上下文
    pop_intr_stack();
    // 在使用iret恢复CPU入栈的上下文
    unsafe {
        asm!("iretd")
    }
}




