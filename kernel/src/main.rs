#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]


use core::{arch::asm, panic::PanicInfo};
use core::mem::size_of;
use device::{Disk, Partition};
use filesystem::{File, FileType, OpenOptions};
use kernel::filesystem::SeekFrom;
use kernel::{console_println, device, filesystem, init, memory, println, process, shell, sys_call, thread, thread_management};
use os_in_rust_common::domain::LbaAddr;
use os_in_rust_common::{cstring_utils, disk, instruction, utils, vga, MY_PANIC};
use os_in_rust_common::{ASSERT, context::BootContext, printk, printkln};


static text: &'static str = "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";

const LEN: usize = 10000;

#[inline(never)]
fn test_write_read_file() {
    // 创建一个文件
    let file = File::create("/a.txt");
    printkln!("create result: {:?}", file);
    ASSERT!(file.is_ok());
    let mut file = file.unwrap();
    // 写入数据
    let write_res = file.write(text.as_bytes());
    printkln!("write file res: {:?}", write_res);

    // 再写入数据
    let write_res = file.write(text.as_bytes());
    printkln!("write file res:  {:?}", write_res);

    let buff: &mut [u8; LEN] = memory::malloc(LEN);
    // 设置偏移量
    printkln!("seek res: {:?}", file.seek(SeekFrom::Start(55)));

    let file = File::open("/a.txt");
    let file = file.unwrap();
    // 读取数据
    let read_result = file.read(buff);
    printkln!("read result: {:?}", read_result);
    let string = core::str::from_utf8(buff);
    printkln!("string result: {:?}", string.unwrap());
}


#[inline(never)]
fn test_create_dir() {
    
    printkln!("res:{:?}", filesystem::create_dir("/sample"));
    printkln!("mkdir result: {:?}", filesystem::create_dir_all("/dev/proc/"));
    printkln!("folder1 res:{:?}", filesystem::create_dir("/dev/proc/folder1"));
    printkln!("folder2 res:{:?}", filesystem::create_dir("/dev/proc/folder2"));
    printkln!("folder3 res:{:?}", filesystem::create_dir("/dev/proc/folder3"));
    printkln!("folder4 res:{:?}", filesystem::create_dir("/dev/proc/folder4"));
    printkln!("folder5 res:{:?}", filesystem::create_dir("/dev/proc/folder5"));
    printkln!("folder6 res:{:?}", filesystem::create_dir("/dev/proc/folder6"));
    printkln!("folder7 res:{:?}", filesystem::create_dir("/dev/proc/folder7"));
    printkln!("folder8 res:{:?}", filesystem::create_dir("/dev/proc/folder8"));
    printkln!("folder9 res:{:?}", filesystem::create_dir("/dev/proc/folder9"));
    printkln!("folder10 res:{:?}", filesystem::create_dir("/dev/proc/folder10"));
    printkln!("folder11 res:{:?}", filesystem::create_dir("/dev/proc/folder11"));
    printkln!("folder12 res:{:?}", filesystem::create_dir("/dev/proc/folder12"));
    printkln!("folder13 res:{:?}", filesystem::create_dir("/dev/proc/folder13"));
    printkln!("folder14 res:{:?}", filesystem::create_dir("/dev/proc/folder14"));
    printkln!("folder15 res:{:?}", filesystem::create_dir("/dev/proc/folder15"));
    printkln!("folder16 res:{:?}", filesystem::create_dir("/dev/proc/folder16"));
    printkln!("folder17 res:{:?}", filesystem::create_dir("/dev/proc/folder17"));
    printkln!("folder18 res:{:?}", filesystem::create_dir("/dev/proc/folder18"));
    printkln!("folder19 res:{:?}", filesystem::create_dir("/dev/proc/folder19"));
    printkln!("folder20 res:{:?}", filesystem::create_dir("/dev/proc/folder20"));

    // 删除一个文件夹   
    printkln!("remove folder1 res:{:?}", filesystem::remove_dir("/dev/proc/folder1"));
}


#[inline(never)]
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {

    init::init_all(boot_info);
    // self::test_create_dir();
    // self::test_read_dir_entry();
    // self::test_create_file();

    // 写入和读取文件
    // self::test_write_read_file();


    // 读取并且写入用户进程
    read_and_write_user_process();

    // shell::shell_start();

    
    loop {
        thread_management::thread_yield();
    }
}

#[inline(never)]
fn read_and_write_user_process() {
    const FILE_SIZE: u32 = 4608;
    let sec_cnt = utils::div_ceil(FILE_SIZE, 512);
    let channel_idx = 0;
    let channel = device::get_ata_channel(&channel_idx);
    ASSERT!(channel.is_some());
    let channel = channel.as_mut();
    let channel = channel.unwrap();
    let disk = channel.disks[0].as_mut();
    let disk = disk.unwrap();
    let buff: &mut [u8; FILE_SIZE as usize] = memory::malloc(FILE_SIZE.try_into().unwrap());
    disk.read_sectors(LbaAddr::new(300), sec_cnt.try_into().unwrap(), buff);

    let file = File::create("/userproc");
    if file.is_err() {
        MY_PANIC!("failed to create file. error: {:?}", file.unwrap_err());
        return;
    }
    let mut file = file.unwrap();
    file.write(buff);
    memory::sys_free(buff.as_ptr() as usize);
    printkln!("finish write");
}


/**
 * 做一个假的sleep
 */
fn dummy_sleep(instruction_cnt: u32) {
    for _ in 0 .. instruction_cnt {
        unsafe {asm!("nop");}
    }
}




#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printkln!("panic, {}", info);
    loop {}
}