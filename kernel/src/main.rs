#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

mod interrupt;
mod init;
mod thread_management;
mod scheduler;
mod sync;
mod mutex;
mod console;
mod keyboard;
mod scancode;
mod printer;
pub mod blocking_queue;
pub mod tss;
mod memory;
pub mod process;
mod thread;
mod sys_call;
mod pid_allocator;
mod page_util;
mod device;
mod filesystem;


use core::{arch::asm, mem::size_of, panic::PanicInfo};
use device::ata::{Disk, Partition};
use filesystem::inode::OpenedInode;
use filesystem::{fs, DirEntry, File, FileType, OpenOptions, SeekFrom};
use memory::mem_block::{Arena, MemBlock};
use os_in_rust_common::{cstring_utils, instruction, vga};
use os_in_rust_common::{ASSERT, constants, context::BootContext, elem2entry, printk, printkln};
use os_in_rust_common::linked_list::{LinkedList, LinkedNode};
use thread::ThreadArg;


static PROCESS_NAME: &str = "user process";
// static text: &'static str = "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";

// #[inline(never)]
fn test_read_dir_entry() {
    let dir = filesystem::read_dir("/dev/proc/");
    let mut dir = dir.unwrap();
    console_println!("read dir: {:?}", dir.get_path());
    let iterator = dir.iter();
    // printkln!("read dir: {:?}", iterator);
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
    printkln!("create test.txt: {:?}", File::create("/test.txt"));
    // 重复创建，失败
    printkln!("{:?}", File::create("/test.txt"));

    // 创建一个父目录不存在的文件
    printkln!("{:?}", File::create("/dev/proc/test.rs"));

    printkln!("create a.txt: {:?}", File::create("/a.txt"));
    printkln!("create b.txt: {:?}", File::create("/b.txt"));

    filesystem::create_dir_all("/dev/proc");
    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));
    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));

    let fd_table = &thread::current_thread().task_struct.fd_table;
    printkln!("fd table:{}", fd_table);
}

fn print_opened_inode() {
    let fs = filesystem::fs::get_filesystem();
    printk!("opened inode: ");
    fs.open_inodes.iter().for_each(|inode_tag|{
        let inode = OpenedInode::parse_by_tag(inode_tag);
        printk!("{} ", inode.i_no);
    });
    printkln!();
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
}

#[inline(never)]
#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {

    init::init_all(boot_info);
    self::test_create_dir();
    self::test_read_dir_entry();
    self::test_create_file();
    let fs = fs::get_filesystem();
    for node_tag in fs.open_inodes.iter() {
        let inode = OpenedInode::parse_by_tag(node_tag);
        let entry = filesystem::current_inode_entry(inode);
        if entry.is_empty() {
            continue;
        }
        // console_println!("inode_no{}, open_cnt:{}", inode.i_no, inode.open_cnts);
        printkln!("inode: {}, name:{}, open_cnt:{}", inode.i_no, entry.get_name(), inode.open_cnts);
    }

    
    // 打印线程信息
    // thread_management::print_thread();

    printkln!("-----system started-----");
    // let cur_task = &thread::current_thread().task_struct;
    // process::process_execute(PROCESS_NAME, u_prog_a);
    // thread_management::thread_start("thread_a", 5, kernel_thread, 0);

    // instruction::enable_interrupt();

    loop {}
    // 主通道。挂在2个硬盘
    let channel_idx = 0;
    let primary = device::init::get_ata_channel(&channel_idx);
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


fn hello() {
    let hello = b"Hello, World";
    let vga_buffer = 0xC00b8000 as *mut u8;

    for (i, &e) in hello.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = e;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

#[derive(Debug)]
struct StructDTO {
    id: u32,
    tag: LinkedNode,
}

fn test_linked_list() {
    let mut linked_list = LinkedList::new();
    let mut s1 = StructDTO {id: 1, tag: LinkedNode::new()};
    linked_list.push(&mut s1.tag);

    printkln!("list size:{}", linked_list.size());

    let mut s2 = StructDTO {id: 2, tag: LinkedNode::new()};
    linked_list.append(&mut s2.tag);

    let mut s3 = StructDTO {id: 3, tag: LinkedNode::new()};
    linked_list.append(&mut s3.tag);

    let s11 = linked_list.pop();

    let mut s4 = StructDTO {id: 4, tag: LinkedNode::new()};
    linked_list.append(&mut s4.tag);


    linked_list.iter().for_each(|node| {
        let dto = unsafe{ &*elem2entry!(StructDTO, tag, node as usize)};
        printkln!("cur addr:0x{:x}, {:?}", node as usize, *dto);
    })

}

/**
 * 内核线程
 */
extern "C" fn kernel_thread(arg: ThreadArg) {
    let pid = sys_call::get_pid();
    printkln!("kernel thread pid:{}", pid);

    printkln!("size of: 0x{:x}", size_of::<Arena>());
    
    let page_size: usize = 4 * 1024;
    let addr1 = memory::malloc_kernel_page(1);
    printkln!("page addr: 0x{:x}", addr1);

    // 分配1块，新页
    let addr2 = memory::sys_malloc(33);
    printkln!("addr: 0x{:x}, {}", addr2, addr2 == addr1 + size_of::<Arena>() + page_size);

    // 分配1块，新页
    let addr3 = memory::sys_malloc(12);
    printkln!("addr: 0x{:x}, {}", addr3, addr3 == addr2 + page_size);
    
    // 分配2页。新页
    let addr4 = memory::sys_malloc(4096);
    printkln!("addr: 0x{:x}, {}", addr4, addr4 == addr3 +  page_size);
    
    // 分配1块，新页
    let addr5 = memory::sys_malloc(129);
    printkln!("addr: 0x{:x}, {}", addr5,  addr5 == addr4 + 2 * page_size);

    // 分配1块，旧页
    let addr6 = memory::sys_malloc(33);
    printkln!("addr: 0x{:x}, {}", addr6, addr6 == addr2 + 64);

    printk!("size: ");
    memory::mem_block::get_kernel_mem_block_allocator().print_container_size();


    memory::sys_free(addr2);
    printk!("size: ");
    memory::mem_block::get_kernel_mem_block_allocator().print_container_size();

    memory::sys_free(addr6);
    
    printk!("size: ");
    memory::mem_block::get_kernel_mem_block_allocator().print_container_size();

    // memory::mem_block::get_kernel_mem_block_allocator().print_container();

    loop {
        // console_print!("k");
        // dummy_sleep(10000);
    }
}

pub fn test_malloc_free() {

    // 申请10个字节的空间
    let addr1 = memory::sys_malloc(10);
    let container = memory::mem_block::get_kernel_mem_block_allocator().match_container(10);
    let total_size = constants::PAGE_SIZE as usize / container.block_size() - 1;
    assert_true(container.size() ==  total_size - 1, "malloc error");
    
    // 再申请一个10字节
    let addr2 = memory::sys_malloc(10);
    assert_true(container.size() ==  total_size- 2, "malloc error");

    // 释放addr1
    memory::sys_free(addr1);
    assert_true(container.size() ==  total_size - 1, "free error");
    
    // 释放addr2
    memory::sys_free(addr2);
    // 这一页都释放了
    assert_true(container.size() ==  0, "free error");

    // 两个都释放了，那么按理说PTE的P位应该已经清除了
    let arena = unsafe { & *(addr2 as *const MemBlock) }.arena_addr();
    let pde = unsafe { &*page_util::addr_to_pde(arena as *const _ as usize) };
    let pte = unsafe { &*page_util::addr_to_pte(arena as *const _ as usize) };

    // 确保PDE存在
    assert_true(pde.present(), "SYSTEM ERROR, PDE ERROR");
    // 确保PTE不存在
    assert_true(!pte.present(), "ERROR: PTE should be expelled");

}

fn assert_true(condition: bool, msg: &str) {
    if condition {
        return;
    }
    printkln!("{}", msg);
}

#[derive(Debug)]
struct MyStruct {
    id: u32,
    age: u8
}
/**
 * 用户进程
 */
extern "C" fn u_prog_a() {
    let pid = sys_call::get_pid();
    println!("user process pid: {}", pid);
    
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