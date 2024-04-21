#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
mod protect_mode;
use os_in_rust_common::{dap, selector::{SegmentSelector}, println, vga::{self, CharAttr, Color, ScreenBuffer, Writer}};
/**
 * loader的地址，加载到0x900
 */
static LOADER_ADDR: u32 = 0x1000;

/**
 * loader占用的扇区数量，4个扇区
 */
static LOADER_SECTOR_CNT: u16 = 4;

/**
 * loader所在硬盘的LBA地址
 */
static LOADER_LBA: u64 = 6;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    // 取Loader加载到内存地址的高16位
    let loader_seg_addr = (LOADER_ADDR >> 16) as u16;
    // 取loader加载到的内存地址的第16位
    let loader_offset_addr = LOADER_ADDR as u16;

    // 构建Disk Packet Address
    let dap_structre = dap::DiskPacketAddress::new(
        LOADER_LBA,
        LOADER_SECTOR_CNT,
        loader_seg_addr,
        loader_offset_addr,
    );

    // 开始执行，把硬盘的数据加载到内存
    unsafe {
        dap_structre.do_load();
    }

    // let loader_entry: extern "C" fn() = unsafe { core::mem::transmute(LOADER_ADDR as *const ()) };
    protect_mode::enter_protect_mode();
    let selector = SegmentSelector::Code0Selector as u16;
    unsafe {
        asm!(
            "jmp 0x8, 0x1000"
        );
    }

    // loader_entry();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}