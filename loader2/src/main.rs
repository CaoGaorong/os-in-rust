#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{disk, gdt::{self, GlobalDescriptorTable}, paging::{self, PageTable}, println, reg_cr0::{self, CR0}, reg_cr3::CR3, selector};
// use core::fmt::Write;
// use os_in_rust_common::{gdt::GlobalDecriptorTable, interrupt, reg_cr0, selector, vga:: {self, CharAttr, Color, ScreenBuffer, Writer, WRITER}};


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    
    // 填充页目录表。
    paging::fill_dir_directory();
    // 填充内核页表
    paging::fill_kernel_directory();
    // 填充0号页表。低端1MB
    paging::fill_table0();

    // 取出GDT的地址
    let gdt_addr = gdt::get_gdt_addr();
    // GDT的新地址。高地址
    let new_gdt_addr = gdt_addr as u32 + 0xc0000000;
    
    
    // 加载到cr3寄存器
    let cr3 = CR3::new(paging::get_dir_ref() as  *const PageTable);
    cr3.load_cr3();


    // 打开CR0寄存器的PG位
    reg_cr0::set_on(CR0::PG);

    // 重新加载gdt
    gdt::load_gdtr_by_addr(new_gdt_addr as *const GlobalDescriptorTable);

    // 加载内核
    disk::read_disk(7, 200, 0xc0001500);

    unsafe {
        asm!(
            "jmp 0x8, 0xc0001500"
        );
    }
    // loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}