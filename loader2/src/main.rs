#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{context::{self, BootContext}, disk, gdt::{self, GlobalDescriptorTable}, mem, paging::{self, PageTable}, println, racy_cell::RacyCell, reg_cr0::{self, CR0}, reg_cr3::CR3, selector};
// use core::fmt::Write;
// use os_in_rust_common::{gdt::GlobalDecriptorTable, interrupt, reg_cr0, selector, vga:: {self, CharAttr, Color, ScreenBuffer, Writer, WRITER}};


static BOOT_CONTEXT: RacyCell<BootContext> = RacyCell::new(BootContext {
    memory_map_addr: 0,
    memory_map_len: 0,
});


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: u32) {
    // reg_cr0::set_off(reg_cr0::CR0::PE);
    // let result = unsafe {mem::query_memory_map()};
    // match result {
    //     Ok(_) => boot_info = 20,
    //     Err(_) => boot_info = 1,
    // }
    // reg_cr0::set_on(reg_cr0::CR0::PE);

    // let context = context::BootContext { 
    //     memory_map_addr: memeory_map.as_mut_ptr() as u32, 
    //     memory_map_len: memeory_map.len() as u32, 
    // };
    // *unsafe { BOOT_CONTEXT.get_mut() } = context;


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
        // 跳转，使用ATT风格
        asm!("jmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            ".code32",
            "mov esp, 0xc009f000",
            "push {0:e}",
            "push 0xc0001500",
            "pop {1:e}",
            "call {1:e}",
            in(reg) boot_info,
            out(reg) _,
           
        );
    }
    // loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}