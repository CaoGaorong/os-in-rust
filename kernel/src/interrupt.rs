use core::{arch::asm, ptr::addr_of};

use os_in_rust_common::{idt::{self, HandlerFunc, InterruptStackFrame, InterruptTypeEnum}, instruction, pic, print, println};


pub fn init() {
    // 初始化时钟中断
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Timer, general_handler) }
    
    idt::idt_init();

    // 初始化中断控制器
    pic::init_pic();

}


/**
 * 通用的中断处理程序
 */
extern "x86-interrupt" fn general_handler(frame: InterruptStackFrame) {
    print!("interrupt occur");
    pic::send_end_of_interrupt();
}




