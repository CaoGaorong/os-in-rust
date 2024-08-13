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
use kernel::sys_call::sys_call_proxy;
use kernel::{console_println, device, filesystem, init, println, process, sys_call, thread};
use os_in_rust_common::{cstring_utils, instruction, vga};
use os_in_rust_common::{ASSERT, context::BootContext, printk, printkln};


static PROCESS_NAME: &str = "init";
// static text: &'static str = "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";

#[inline(never)]
fn test_read_dir_entry() {
    let dir = filesystem::read_dir("/dev/proc/");
    let mut dir = dir.unwrap();
    console_println!("read dir: {:?}", dir.get_path());
    let iterator = dir.iter();
    for dir_entry in  iterator {
        console_println!("entry_name: {:?}, file_type: {:?}", dir_entry.get_name(), dir_entry.file_type as FileType);
    }
}

#[inline(never)]
fn test_write_read_file() {
    // 打开一个文件
    let file = OpenOptions::new().read(true).write(true).open("/a.txt");
    printkln!("open_result: {:?}", file);
    ASSERT!(file.is_ok());
    let mut file = file.unwrap();
    // 写入数据
    // let write_res = file.write(text.as_bytes());
    // printkln!("write file res: {:?}", write_res);

    // // 再写入数据
    // let write_res = file.write(text.as_bytes());
    // printkln!("write file res:  {:?}", write_res);

    // let buff: &mut [u8; 666] = memory::malloc(666);
    // // 设置偏移量
    // printkln!("seek res: {:?}", file.seek(SeekFrom::Start(55)));

    // // 读取数据
    // let read_result = file.read(buff);
    // printkln!("read result: {:?}", read_result);
    // let string = core::str::from_utf8(buff);
    // printkln!("string result: {:?}", string.unwrap());
}

#[inline(never)]
fn test_create_file() {
    // 创建文件
    let res = File::create("/test.txt");
    printkln!("create test.txt, path:{}", res.unwrap().get_path());
    // 重复创建，失败
    printkln!("{:?}", File::create("/test.txt"));

    // 创建一个父目录不存在的文件
    printkln!("{:?}", File::create("/dev/proc/abc/test.rs"));

    printkln!("create a.txt: {:?}", File::create("/a.txt"));
    printkln!("create b.txt: {:?}", File::create("/b.txt"));

    filesystem::create_dir_all("/dev/proc");
    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));
    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));

    let fd_table = &thread::current_thread().task_struct.fd_table;
    printkln!("fd table:{}", fd_table);

    printkln!("remove a.txt: {:?}", filesystem::remove_file("/a.txt"));
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
extern "C" fn init_process() {

    // 发起系统调用，申请内存空间
    let my_struct_ptr: *mut MyStruct = sys_call::malloc(size_of::<MyStruct>());
    let my_struct:&mut MyStruct =  unsafe { &mut *my_struct_ptr };
    my_struct.id = 10;
    my_struct.age = 18;

    println!("{:?}", my_struct); // 正常打印
    
    let fork_res = sys_call_proxy::fork();
    match fork_res {
        sys_call_proxy::ForkResult::Parent(child_pid) => {
            println!("i'm father, my pid is {}, my child pid is {}", sys_call_proxy::get_pid().get_data(), child_pid.get_data());
        },
        sys_call_proxy::ForkResult::Child => {
            println!("im child, my pid is {}", sys_call_proxy::get_pid().get_data());
        },
    }
    println!("finish fork");


    // 释放内存空间
    sys_call::free(my_struct_ptr);
    loop {}
}


#[inline(never)]
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {

    init::init_all(boot_info);
    // self::test_create_dir();
    // self::test_read_dir_entry();
    // self::test_create_file();

    // let fs = fs::get_filesystem();
    // fs.iter_open_nodes(|inode| {
    //     let entry = filesystem::current_inode_entry(inode);
    //     if entry.is_empty() {
    //         return;
    //     }
    //     // console_println!("inode_no{}, open_cnt:{}", inode.i_no, inode.open_cnts);
    //     printkln!("inode: {}, name:{}, open_cnt:{}", inode.i_no, entry.get_name(), inode.open_cnts);
    // });

    // let cur_task = &mut thread::current_thread().task_struct;

    // // 更改工作目录
    // filesystem::change_dir(cur_task, "/dev/proc/");
    // let mut buf: [u8; 10] = [0; 10];
    // filesystem::get_cwd(cur_task, &mut buf);
    // let cwd = cstring_utils::read_from_bytes(&buf);
    // printkln!("cwd:{:?}", cwd.unwrap());
    
    // 打印线程信息
    // thread_management::print_thread();

    printkln!("-----system started-----");
    // let cur_task = &thread::current_thread().task_struct;
    process::process_execute(PROCESS_NAME, init_process);
    // thread_management::thread_start("thread_a", 5, kernel_thread, 0);

    // loop {}
    // 主通道。挂在2个硬盘
    let channel_idx = 0;
    let primary = device::get_ata_channel(&channel_idx);
    ASSERT!(primary.is_some());
    let primary = primary.as_mut().unwrap();
    // 次通道。没硬盘
    // let secondary = device::init::get_ata_channel(1);
    printkln!("primary channel: ");
    let channel_name = cstring_utils::read_from_bytes(&primary.name);
    printk!("name:{}, port_base:0x{:x}, irq_no: 0x{:x} ", channel_name.unwrap(), primary.port_base, primary.irq_no);
    printkln!("disk[0] ignored. disk[1]:");
    let disk =  &mut primary.disks[1];
    print_disk(disk.as_ref().unwrap());

    instruction::enable_interrupt();
    
    // // 测试一样空间的分配和释放
    // test_malloc_free();

    // 测试链表
    // test_linked_list();


    
    loop {}
}

fn print_disk(disk: &Disk) {
    let disk_name =  cstring_utils::read_from_bytes(&disk.name);
    ASSERT!(disk_name.is_some());
    printkln!("name:{}, from_channel:{}, is_primary:{}", disk.get_name(), disk.from_channel as usize, disk.primary);

    let pp = &disk.primary_parts;
    for (idx, part) in pp.iter().enumerate() {
        printk!("primary part[{}]", idx);
        if part.is_none() {
            printk!(" is none\n");
            continue;
        }
        print_partition(part.as_ref().unwrap());
    }
    
    let lp = &disk.logical_parts;
    for (idx, part) in lp.iter().enumerate() {
        printk!("logical part[{}]", idx);
        if part.is_none() {
            printk!("is none\n");
            continue;
        }
        print_partition(part.as_ref().unwrap());
    }
}

fn print_partition(part: &Partition) {
    // let part_name =  cstring_utils::read_from_bytes(&part.name);
    // ASSERT!(part_name.is_some());
    // printkln!("name:{}, lba:{:?}, sec_cnt:{}, from_disk:{}", part.get_name(), part.abs_lba_start(0), part.sec_cnt, part.from_disk as usize);
    printkln!(" {}, {:?}, {}, {}", part.get_name(), part.abs_lba_start(0), part.sec_cnt, part.from_disk as usize);
}


#[derive(Debug)]
struct MyStruct {
    id: u32,
    age: u8
}
/**
 * 用户进程
 */
#[inline(never)]
extern "C" fn u_prog_a() {
    let pid = sys_call::get_pid();
    println!("user process pid: {}", pid.get_data());
    
    // 发起系统调用，申请内存空间
    let my_struct_ptr: *mut MyStruct = sys_call::malloc(size_of::<MyStruct>());
    let my_struct:&mut MyStruct =  unsafe { &mut *my_struct_ptr };
    my_struct.id = 10;
    my_struct.age = 18;

    println!("{:?}", my_struct); // 正常打印

    // 释放内存空间
    sys_call::free(my_struct_ptr);

    loop {
        // print!("u");
        // dummy_sleep(10000);
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
    instruction::disable_interrupt();
    printkln!("panic, {}", info);
    let msg = info.message();
    if msg.is_none() {
        loop {}
    }
    vga::print(*info.message().unwrap());
    loop {}
}