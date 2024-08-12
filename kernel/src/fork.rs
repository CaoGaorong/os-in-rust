use core::{mem, ptr};
use core::mem::size_of;

use os_in_rust_common::{constants, linked_list::LinkedNode, paging::PageTable, printkln, ASSERT};

use crate::process;
use crate::{filesystem::{self}, memory::{self, MemBlockAllocator}, pid_allocator::{self, Pid}, thread::{self, PcbPage, TaskStatus, TaskStruct}, thread_management};


#[inline(never)]
pub fn fork() -> Pid {
    // 当前PCB
    let cur_pcb: &PcbPage = thread::current_thread();
    
    // 确保是用户进程在调用，而不是内核线程
    ASSERT!(!cur_pcb.task_struct.pgdir.is_null());

    thread::check_task_stack("failed to fork");

    // 申请一个空间，子任务的PCB
    let sub_pcb = unsafe { &mut *(memory::malloc_kernel_page(1) as *mut PcbPage) };
    
    // 拷贝PCB。浅拷贝，拷贝PCB结构本身
    self::pcb_shallow_copy(cur_pcb, sub_pcb);
    thread::check_task_stack("failed to fork, pcb copy error");
    
    // 申请1页作为页表
    sub_pcb.task_struct.pgdir = process::create_page_dir();
    thread::check_task_stack("failed to fork, create page dir error");

    // 拷贝 虚拟地址池。每个TaskStruct有一个虚拟地址池（堆空间）
    self::vaddr_pool_copy(&cur_pcb.task_struct, &mut sub_pcb.task_struct);
    thread::check_task_stack("failed to fork, copy vaddr pool error");
    
    // 拷贝 堆内存（该任务的页表映射了的所有内存）
    self::heap_memory_copy(&mut sub_pcb.task_struct);
    thread::check_task_stack("failed to fork, copy heap memory error");
    
    let sub_task_page_table = unsafe { &*sub_pcb.task_struct.pgdir };
    for (idx, entry) in sub_task_page_table.iter().enumerate() {
        if !entry.present() {
            continue;
        }
        printkln!("present, idx:{}", idx);
    }
    
    // 重新构建子任务的栈（栈内决定了该程序被调度时的执行）
    // self::rebuild_stack(sub_pcb);
    
    // 把打开的文件再打开一次
    // self::reopen_file(&mut sub_pcb.task_struct);

    // ASSERT!(thread::get_all_thread().contains(&sub_pcb.task_struct.all_tag));
    // thread::append_all_thread(&mut sub_pcb.task_struct);

    // ASSERT!(thread::get_ready_thread().contains(&sub_pcb.task_struct.general_tag));
    // thread::append_read_thread(&mut sub_pcb.task_struct);

    // 对于父进程，返回子进程的pid
    return sub_pcb.task_struct.pid;
}


/***
 * 针对PCB页的浅拷贝
 */
#[inline(never)]
fn pcb_shallow_copy(from: &PcbPage, to: &mut PcbPage) {
    // 浅拷贝，逐个bit拷贝
    let from_page =  unsafe { core::slice::from_raw_parts(from as *const _ as *const u8, size_of::<PcbPage>()) };
    let to_page =  unsafe { core::slice::from_raw_parts_mut(to as *mut _ as *mut u8, size_of::<PcbPage>()) };
    to_page.copy_from_slice(from_page);
    
    let from_task = &from.task_struct;
    let to_task = &mut to.task_struct;

    // 申请新的PID
    to_task.pid = pid_allocator::allocate();
    to_task.parent_pid = from_task.pid;
    // 状态为就绪
    to_task.task_status = TaskStatus::TaskReady;
    to_task.elapsed_ticks = 0;
    to_task.pgdir = ptr::null_mut();
    to_task.general_tag = LinkedNode::new();
    to_task.all_tag = LinkedNode::new();
    to_task.mem_block_allocator = MemBlockAllocator::new();
}

/**
 * 虚拟地址池拷贝
 */
#[inline(never)]
fn vaddr_pool_copy(from_task: &TaskStruct, to_task: &mut TaskStruct) {
    // 申请堆空间，作为用户地址位图
    to_task.vaddr_pool = thread_management::apply_user_addr_pool();


    let from_bitmap = unsafe { core::slice::from_raw_parts( from_task.vaddr_pool.bitmap.map_ptr, from_task.vaddr_pool.bitmap.size) };
    let to_bitmap = unsafe { core::slice::from_raw_parts_mut( to_task.vaddr_pool.bitmap.map_ptr, to_task.vaddr_pool.bitmap.size) };
    // 把虚拟地址池，指向的内容拷贝一下
    to_bitmap.copy_from_slice(from_bitmap);
}

/**
 * 堆内存拷贝
 */
#[inline(never)]
fn heap_memory_copy(to_task: &mut TaskStruct) {
    let from_task = &thread::current_thread().task_struct;
    let to_task_dir_table = &mut (unsafe { *to_task.pgdir });

    // 要拷贝的任务的虚拟地址池
    let from_task_addr_pool = &from_task.vaddr_pool;

    // 作为复制页表的入参，默认是None
    let mut page_table_req = Option::None;
    // 遍历 虚拟地址池，里面所有被set过的地址
    for (vaddr, set) in from_task_addr_pool.iter_valid() {
        // 这个地址没有被使用，那么不需要
        if !set {
            continue;
        }
        // 这个地址指向页的数据
        let page_data = unsafe { core::slice::from_raw_parts(vaddr as *const u8, constants::PAGE_SIZE as usize) };

        // 把这个页的数据，拷贝到另一个任务的页目录表中。得到操作的页表，用于下次循环
        let page_table  = memory::copy_single_user_page(page_data, to_task_dir_table, page_table_req);

        // 把得到的页表，作为下次循环的参数
        page_table_req = Option::Some(page_table);
    }
}

/**
 * 重新构建栈，因为当一个程序被调度（schedule处理中从就绪列表取出一个任务，然后执行），
 * 这个程序的执行路线完全由栈内的数据决定（通过switch_to函数的退出的ret指令，把栈的数据弹出，并且执行）
 */
#[inline(never)]
fn rebuild_stack(pcb: &mut PcbPage) {
    // 对于子进程，返回值为0。
    pcb.interrupt_stack.eax = 0;

    // 初始化程序退出的线程栈。当该任务被执行（该任务被switch_to调度到后执行）
    pcb.init_exit_thread_stack();
}

/**
 * 把这个任务打开的文件，再打开一次（父进程打开了一次，子进程也要打开一次）
 */
#[inline(never)]
fn reopen_file(task: &mut TaskStruct) {
    let file_descriptors = task.fd_table.get_file_descriptors();
    if file_descriptors.is_empty() {
        return;
    }
    for descriptor in file_descriptors {
        if descriptor.is_none() {
            continue;
        }
        let opened_file = filesystem::get_opened_file(descriptor.unwrap());
        if opened_file.is_none() {
            continue;
        }
        let opened_file = opened_file.unwrap();
        // 再次打开一次
        opened_file.reopen();
    }
}