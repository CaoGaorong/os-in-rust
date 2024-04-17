#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os_in_rust_common::vga:: {Writer, CharAttr, Color, ScreenBuffer};
use os_in_rust_common::sd::SegmentDescritor;
use os_in_rust_common::gdt::GlobalDecriptorTable;

static GDT: GlobalDecriptorTable = GlobalDecriptorTable::new();

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    // 加载GDTR
    GDT.load_gdtr();
    
    
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}