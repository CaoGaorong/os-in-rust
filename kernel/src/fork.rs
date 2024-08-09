use core::ptr;
use core::mem::size_of;

use os_in_rust_common::{constants, linked_list::LinkedNode, paging::PageTable, ASSERT};

use crate::{filesystem::FileDescriptorTable, memory::{self, mem_block::MemBlockAllocator}, pid_allocator::{self, Pid}, thread::{self, PcbPage, TaskStatus, TaskStruct}, thread_management};


pub fn fork() -> Pid {
    // 当前PCB
    let cur_pcb = thread::current_thread();
    
    // 确保是用户进程在调用，而不是内核线程
    ASSERT!(cur_pcb.task_struct.pgdir.is_null());

    // 申请一个空间，子任务的PCB
    let sub_pcb = unsafe { &mut *(memory::malloc_kernel_page(1) as *mut PcbPage) };
    
    // 拷贝PCB。浅拷贝
    self::pcb_copy(cur_pcb, sub_pcb);

    let from_task = &cur_pcb.task_struct;
    let to_task = &mut sub_pcb.task_struct;

    // 申请1页作为页表
    to_task.pgdir = memory::malloc_kernel_page(1) as *mut PageTable;

    // 拷贝 虚拟地址池
    self::vaddr_pool_copy(from_task, to_task);
    
    // 拷贝堆内存（页表指向的内存）
    self::heap_memory_copy(to_task);

    return to_task.pid;

}

fn pcb_copy(from: &PcbPage, to: &mut PcbPage) {
    // 浅拷贝
    unsafe { ptr::copy(from as *const _, to as *mut _, size_of::<PcbPage>()) }
    let from_task = &from.task_struct;
    let to_task = &mut to.task_struct;
    
    // 申请新的PID
    to_task.pid = pid_allocator::allocate();
    to_task.parent_pid = from_task.pid;
    // 状态为就绪
    to_task.task_status = TaskStatus::TaskReady;
    to_task.elapsed_ticks = 0;
    to_task.pgdir = ptr::null_mut();
    to_task.fd_table = FileDescriptorTable::new();
    to_task.general_tag = LinkedNode::new();
    to_task.all_tag = LinkedNode::new();
    to_task.mem_block_allocator = MemBlockAllocator::new();
}

/**
 * 虚拟地址池拷贝
 */
fn vaddr_pool_copy(from_task: &TaskStruct, to_task: &mut TaskStruct) {
    // 申请堆空间，作为用户地址位图
    to_task.vaddr_pool = thread_management::apply_user_addr_pool();

    // 把虚拟地址池，指向的内容拷贝一下
    unsafe { core::ptr::copy(from_task.vaddr_pool.bitmap.map_ptr, to_task.vaddr_pool.bitmap.map_ptr, to_task.vaddr_pool.bitmap.size) };
}

/**
 * 堆内存拷贝
 */
fn heap_memory_copy(to_task: &mut TaskStruct) {
    let from_task = &thread::current_thread().task_struct;
    let to_task_dir_table = &mut (unsafe { *to_task.pgdir });
    
    // 要拷贝的任务的虚拟地址池
    let from_task_addr_pool = &from_task.vaddr_pool;

    // 作为复制页表的入参，默认是None
    let mut page_table_req = Option::None;
    
    // 遍历 虚拟地址池，里面所有被set过的地址
    for (vaddr, set) in from_task_addr_pool.iter() {
        // 这个地址没有被使用，那么不需要
        if !set {
            continue;
        }
        // 这个地址指向页的数据
        let page_data = unsafe { core::slice::from_raw_parts(vaddr as *const u8, constants::PAGE_SIZE as usize) };
        
        // 把这个页的数据，拷贝到另一个任务的页目录表中。得到操作的页表，用于下次循环
        let page_table  = memory::copy_single_page(page_data, to_task_dir_table, page_table_req);
        
        // 把得到的页表，作为下次循环的参数
        page_table_req = Option::Some(page_table);
    }
}