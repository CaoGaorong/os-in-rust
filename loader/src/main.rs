#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
mod protect_mode;
use os_in_rust_common::{constants, context::{self, BootContext}, dap, disk, instruction, mem, println, racy_cell::RacyCell, selector::SegmentSelector};

// #[no_mangle]
static BOOT_CONTEXT: RacyCell<BootContext> = RacyCell::new(BootContext::empty());

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {

    // instruction::disable_interrupt();
    let context = unsafe { BOOT_CONTEXT.get_mut() };
    let memeory_map = unsafe {mem::query_memory_map()};
    context.memory_map_len = memeory_map.0 as u32;

    // match memeory_map {
    //     Ok(adrs_list) => {
    //         // let ptr = adrs_list.as_ptr();
    //         context.memory_map_addr = adrs_list.0 as u32;
    //         context.memory_map_len = adrs_list.1 as u32;
    //     },
    //     Err(_) => {
    //         context.memory_map_addr = 0x456;
    //         context.memory_map_len = 0x456;
    //     },
    // }


    // context.memory_map_addr = memeory_map.as_ptr() as u32;
    // context.memory_map_len = memeory_map.len() as u32;
    // match memeory_map {
    //     Ok(_) => boot_context.memory_map_len = 0x10,
    //     Err(_) => boot_context.memory_map_len = 0x20,
    // }


    // 进入保护模式
    protect_mode::enter_protect_mode();
    
    // 把loader2从磁盘加载到内存
    disk::read_disk(constants::LOADER2_LBA, constants::LOADER2_SEC_CNT as u8, constants::LOADER2_ADDR);

    // let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(0xc00 as *const ()) };
    // let selector = SegmentSelector::Code0Selector as u16;

    unsafe {
        // 跳转，使用ATT风格
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            ".code32",
            "push {0:e}",
            "push 0xc00",
            "pop {1:e}",
            "call {1:e}",
            in(reg) context as *const _  as u32,
            out(reg) _,
        );
    }

    // loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}