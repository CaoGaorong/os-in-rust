#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
mod protect_mode;
use os_in_rust_common::{bios_mem, constants, context::{self, BootContext}, dap, disk, instruction, println, racy_cell::RacyCell, selector::SegmentSelector};


static BOOT_CONTEXT: RacyCell<BootContext> = RacyCell::new(BootContext::empty());

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // instruction::disable_interrupt();
    
    // 调用BIOS，得到内存图
    let result = bios_mem::get_memeory_map();
    let context = unsafe { BOOT_CONTEXT.get_mut() };
    context.memory_map_addr = result.0;
    context.memory_map_len = result.1;

    // 进入保护模式
    protect_mode::enter_protect_mode();
    
    // 把loader2从磁盘加载到内存
    disk::read_disk(constants::LOADER2_LBA, constants::LOADER2_SEC_CNT as u8, constants::LOADER2_ADDR);

    unsafe {
        // 跳转，使用ATT风格
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            // ".code32",
            "push bx",
            "call cx",
            in("cx") constants::LOADER2_ADDR,
            in("bx") context as *const _  as u16,
        );
    }

    // loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}