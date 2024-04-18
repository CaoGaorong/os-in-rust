#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_in_rust_common::vga:: {Writer, CharAttr, Color, ScreenBuffer};
use os_in_rust_common::sd::SegmentDescritor;
use os_in_rust_common::gdt::{self, GlobalDecriptorTable};

static GDT: GlobalDecriptorTable = GlobalDecriptorTable::new();

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // 构造GDT
    let gdtr = GDT.compose_gdtr();
    // 加载GDT到GDTR寄存器中
    gdt::load_gdtr(&gdtr);

    // 打印一下
    let mut writer = Writer::new(unsafe { &mut *(0xb8000 as *mut ScreenBuffer) }, CharAttr::new(Color::White, Color::Black, false));
    writer.write_string("Fuck you");

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}