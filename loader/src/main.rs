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
    
    let context = unsafe { BOOT_CONTEXT.get_mut() };

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
            in("cx") 0xc00,
            in("bx") context as *const _  as u16,
        );
    }

}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}