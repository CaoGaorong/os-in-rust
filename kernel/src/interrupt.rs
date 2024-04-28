use core::arch::asm;

use os_in_rust_common::{idt::{self, HandlerFunc, InterruptStackFrame, InterruptTypeEnum}, instruction, pic, println};

#[used]
#[no_mangle]
static GENERAL_HANDLER:HandlerFunc = handler;

pub fn init() {
    // 初始化idt
    unsafe { idt::IDT.get_mut().set_handler(InterruptTypeEnum::Timer, GENERAL_HANDLER) }
    
    idt::idt_init();

    // 初始化中断控制器
    pic::init_pic();

}


#[no_mangle]
extern "x86-interrupt" fn handler(frame: InterruptStackFrame) {
    println!(".");
    // pic::send_end_of_interrupt(InterruptTypeEnum::Timer);
}




