#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]

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


use core::{arch::asm, mem::size_of, panic::PanicInfo};
use memory::mem_block::{Arena, MemBlock};
use os_in_rust_common::{ASSERT, constants, context::BootContext, elem2entry, instruction::enable_interrupt, printk, printkln};
use os_in_rust_common::linked_list::{LinkedList, LinkedNode};
use thread::ThreadArg;


static PROCESS_NAME: &str = "user process";

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: &BootContext) {
    // hello();
    printkln!("I'm Kernel!");
    
    init::init_all(boot_info);
    
    // 打印线程信息
    // thread_management::print_thread();

    process::process_execute(PROCESS_NAME, u_prog_a);
    // thread_management::thread_start("thread_a", 5, kernel_thread, 0);

    printkln!("-----system started-----");
    printkln!();

    // // 测试一样空间的分配和释放
    // test_malloc_free();

    // 测试链表
    // test_linked_list();


    // enable_interrupt();
    loop {}
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
    
    println!("{:?}", my_struct); // 得到垃圾值

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
    loop {}
}