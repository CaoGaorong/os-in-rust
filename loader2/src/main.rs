#![no_std]
#![no_main]

mod page_table;

use core::{arch::asm, panic::PanicInfo};

use os_in_rust_common::{context::BootContext, disk, gdt::{self, GlobalDescriptorTable}, paging::PageTable, printkln, racy_cell::RacyCell, reg_cr0::{self, CR0}, reg_cr3::CR3};


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
    disk::read_disk(7, 200, 0xc0001500);

    // unsafe {
    //     asm!(
    //         "push eax",
    //         "add dword ptr [ebp-0x0C],01",
    //         "mov eax,[ebp-0x0C]",

    //         "ret", in("eax") enter_kernel,
    //     )
    // }
    // hello();
    enter_kernel(boot_info);

    // unsafe {
    //     // 跳转，使用ATT风格
    //     asm!("jmp $0x8, $2f", "2:", options(att_syntax));
    //     asm!(
    //         ".code32",
    //         "mov esp, 0xc009f000",
    //         "push {0:e}",
    //         "push 0xc0001500",
    //         "pop {1:e}",
    //         "call {1:e}",
    //         in(reg) boot_info,
    //         out(reg) _,
           
    //     );
    // }
    // loop {}
}

#[no_mangle]
#[inline(always)]
fn enter_kernel(boot_info: &BootContext) {
    let boot_info_addr = boot_info as *const _ as usize;
    unsafe {
        asm!(
            "mov esp, 0xc009f000",
            "push {0:e}",
            "push 0xffffffff",
            "jmp 0x8, 0xc0001500",
            in(reg) boot_info_addr

        )
    }
}

fn hello() {
    let hello = b"Hello, World";
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &e) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = e;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}