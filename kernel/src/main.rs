#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]


use core::{arch::asm, panic::PanicInfo};
use filesystem::File;
use kernel::filesystem::SeekFrom;
use kernel::{filesystem, init, memory, pipe, program_loader, sys_call, thread_management};
use os_in_rust_common::domain::LbaAddr;
use os_in_rust_common::constants;
use os_in_rust_common::{ASSERT, context::BootContext, printkln};
use kernel::println;

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

    // 写入和读取文件
    // self::test_write_read_file();


    // 读取并且写入用户进程
    program_loader::sync_program(LbaAddr::new(250), 100 * constants::DISK_SECTOR_SIZE, "/cat");
    program_loader::sync_program(LbaAddr::new(350), 50 * constants::DISK_SECTOR_SIZE, "/grep");
    program_loader::sync_program(LbaAddr::new(400), 10 * constants::DISK_SECTOR_SIZE, "/echo");
    program_loader::sync_program(LbaAddr::new(410), 20, "/main.rs");

    // println!("Hello, world");
    // println!("fuck world");
    // println!("fuck world");
    // let pipe = pipe::pipe(100);
    // if pipe.is_err() {
    //     printkln!("error:{:?}", pipe.unwrap_err());
    // } else if pipe.is_ok() {
    //     let fd = pipe.unwrap();
    //     sys_call::write(fd, "hello, world".as_bytes());
    //     sys_call::pipe_end(fd);
        
    //     let mut buff = [0u8; 20];
    //     sys_call::read(fd, &mut buff);
    //     printkln!("{}", core::str::from_utf8(&buff).unwrap());
    // }

    loop {
        thread_management::thread_yield();
    }
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