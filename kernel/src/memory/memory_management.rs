
use core::ptr;

use os_in_rust_common::racy_cell::RacyCell;

use crate::{sync::Lock, thread};

use super::{mem_block, memory_allocation, memory_deallocation, memory_poll};



/**
 * 内核地址池的锁
 */
static KERNEL_ADDR_POOL_LOCK: RacyCell<Lock> = RacyCell::new(Lock::new());
/**
 * 内核内存池的锁
 */
static KERNEL_MEM_POOL_LOCK: RacyCell<Lock> = RacyCell::new(Lock::new());
/**
 * 用户内存池的锁
 */
static USER_MEM_POOL_LOCK: RacyCell<Lock> = RacyCell::new(Lock::new());

#[inline(never)]
pub fn malloc_system<T>(bytes: usize) -> &'static mut T {
    let cur_task = &mut thread::current_thread().task_struct;
    let pgdir_bak = cur_task.pgdir;
    cur_task.pgdir = ptr::null_mut();
    let t: &mut T = self::malloc(bytes);
    cur_task.pgdir = pgdir_bak;
    t
}

/**
 * 申请内存
 */
#[inline(never)]
pub fn malloc<T>(bytes: usize) -> &'static mut T {
    let addr = sys_malloc(bytes);
    let ptr = addr as *mut T;
    // 返回数据
    unsafe { &mut *ptr }
}

/**
 * 是否某一块空间
 *  - vaddr_to_free: 要释放的空间的地址
 */
#[inline(never)]
pub fn sys_free(vaddr_to_free: usize) {
    thread::check_task_stack("failed to free memory");
    // 当前任务
    let task = &mut thread::current_thread().task_struct;
    // 找出物理内存池。内核程序或者用户程序
    if task.pgdir == ptr::null_mut() {
        unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().lock() };
        unsafe { KERNEL_MEM_POOL_LOCK.get_mut().lock() };
        memory_deallocation::free_bytes(memory_poll::get_kernel_addr_pool(), memory_poll::get_kernel_mem_pool(), vaddr_to_free);
        unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().unlock() };
        unsafe { KERNEL_MEM_POOL_LOCK.get_mut().unlock() };
    } else {
        unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
        memory_deallocation::free_bytes(&mut task.vaddr_pool, memory_poll::get_user_mem_pool(), vaddr_to_free);
        unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
    }
}

/**
 * 在内核空间申请bytes字节的空间
 */
#[inline(never)]
pub fn sys_malloc(bytes: usize) -> usize {
    thread::check_task_stack("failed to malloc memory");

    // 当前任务
    let task = &mut thread::current_thread().task_struct;

    // 找出物理内存池。内核程序或者用户程序
    if task.pgdir == ptr::null_mut() {
        unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().lock() };
        unsafe { KERNEL_MEM_POOL_LOCK.get_mut().lock() };
        let bytes = memory_allocation::malloc_bytes(memory_poll::get_kernel_addr_pool(), memory_poll::get_kernel_mem_pool() , mem_block::get_kernel_mem_block_allocator(), bytes);
        unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().unlock() };
        unsafe { KERNEL_MEM_POOL_LOCK.get_mut().unlock() };
        bytes
    } else {
        unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
        let bytes = memory_allocation::malloc_bytes(&mut task.vaddr_pool, memory_poll::get_user_mem_pool(), &mut task.mem_block_allocator, bytes);
        unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
        bytes
    }
}

/**
 * 申请page_cnt个内核页。得到虚拟地址
 */
pub fn malloc_kernel_page(page_cnt: usize) -> usize { 
    thread::check_task_stack("failed to malloc kernel memory");
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().lock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().lock() };
    let bytes = memory_allocation::malloc_page(memory_poll::get_kernel_addr_pool(), memory_poll::get_kernel_mem_pool(), page_cnt);
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().unlock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().unlock() };
    bytes
}

/**
 * 已知栈顶，分配一个物理页
 */
pub fn malloc_user_stack_page(user_stack_top: usize) {
    thread::check_task_stack("failed to malloc user stack memory");
    unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
    memory_allocation::malloc_phy_by_vaddr(user_stack_top, memory_poll::get_user_mem_pool());
    unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
}