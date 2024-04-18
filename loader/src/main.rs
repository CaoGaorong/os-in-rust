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
    
    unsafe {
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset(0 as isize * 2) = b'I';
        *vga_buffer.offset(0 as isize * 2 + 1) = 0xb;
    }
    
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}