#![no_std]
#![no_main]

mod page_table;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{constants, context::BootContext, disk, gdt::{self, GlobalDescriptorTable}, instruction, paging::PageTable, racy_cell::RacyCell, reg_cr0::{self, CR0}, reg_cr3::CR3};


static BOOT_CONTEXT: RacyCell<BootContext> = RacyCell::new(BootContext {
    memory_map_addr: 0,
    memory_map_len: 0,
});


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {

    // 填充页目录表。
    page_table::fill_dir_directory();
    // 填充内核页表
    page_table::fill_kernel_directory();
    // 填充0号页表。低端1MB
    page_table::fill_table0();

    // 取出GDT的地址
    let gdt_addr = gdt::get_gdt_addr();
    // GDT的新地址。高地址
    let new_gdt_addr = gdt_addr as u32 + 0xc0000000;
    
    
    // 加载到cr3寄存器
    let cr3 = CR3::new(page_table::get_dir_ref() as  *const PageTable);
    cr3.load_cr3();


    // 打开CR0寄存器的PG位
    reg_cr0::set_on(CR0::PG);

    // 重新加载gdt
    gdt::load_gdtr_by_addr(new_gdt_addr as *const GlobalDescriptorTable);

    // 加载内核
    disk::read_disk(7, 200, constants::KERNEL_ADDR);

    instruction::disable_interrupt();

    // 使用jmp指令，跳转到内核
    jump_to_kernel(boot_info);
}


/**
 * 跳转进入内核。
 *  - 先设置内核栈顶的地址
 *  - 把内核入口需要的参数入栈
 *  - 使用jmp指令跳转，刷新选择子
 */
#[inline(always)]
fn jump_to_kernel(boot_info: &BootContext) {
    unsafe {
        asm!(
            // 设定栈顶地址
            "mov esp, {0:e}",
            // 把内核入口需要的参数传递
            "push {1:e}",
            // 这里是使用jmp指令模拟call指令，因此这里传入一个无用的值
            // 进入到内核代码后，会认为栈顶是返回地址，栈顶 + 1 是入参
            // 因此这样保证 boot_info 是作为参数传递的
            "push 0x0",
            "jmp 0x8, 0xc0001500",
            in(reg) 0xc009f000u32,
            in(reg) boot_info

        )
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}