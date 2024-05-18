
use core::{arch::asm, ptr::addr_of};

use os_in_rust_common::{constants, idt::{self, HandlerFunc, InterruptStackFrame, InterruptTypeEnum}, instruction, pic, pit, port::Port, print, println, thread, ASSERT};

use crate::{interrupt, keyboard::{self, ScanCodeCombinator}, scheduler};

pub fn init() {
    
    unsafe { idt::IDT.get_mut().set_error_code_handler(InterruptTypeEnum::GeneralProtectionFault, general_protection_handler) }

    // 初始化时钟中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Timer, timer_handler) }
    // 初始化键盘中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Keyboard, keyboard_handler) }
    
    idt::idt_init();

    // 初始化中断控制器
    pic::pic_init();

    pit::pit_init();

}



/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn general_handler(frame: InterruptStackFrame) {
    // print!(".");
    pic::send_end_of_interrupt();
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
/**
 * 
 */
pub extern "x86-interrupt" fn timer_handler(frame: InterruptStackFrame) {
    // println!("interrupt on {}", instruction::is_intr_on());
    pic::send_end_of_interrupt();
    let current_thread = thread::current_thread();
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

    
 }




