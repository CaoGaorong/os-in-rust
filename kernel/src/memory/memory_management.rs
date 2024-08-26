
use core::ptr;

use os_in_rust_common::{constants, paging::{PageTable, PageTableEntry}, pool::MemPool, racy_cell::RacyCell, ASSERT, MY_PANIC};

use crate::{memory::page_util, sync::Lock, thread::{self, TaskStruct}};

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
#[inline(never)]
pub fn malloc_kernel_page(page_cnt: usize) -> usize { 
    // thread::check_task_stack("failed to malloc kernel page memory");
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().lock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().lock() };
    let bytes = memory_allocation::malloc_page(memory_poll::get_kernel_addr_pool(), memory_poll::get_kernel_mem_pool(), page_cnt);
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().unlock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().unlock() };
    bytes
}

#[inline(never)]
pub fn malloc_user_page(task: &mut TaskStruct, page_cnt: usize) -> usize {
    thread::check_task_stack("failed to malloc user page memory");
    unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
    let bytes = memory_allocation::malloc_page(&mut task.vaddr_pool, memory_poll::get_user_mem_pool(), page_cnt);
    unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
    bytes
}

#[inline(never)]
pub fn free_user_page(task: &mut TaskStruct, vaddr: usize, page_cnt: usize, phy_free: bool) {
    unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
    memory_deallocation::free_page(&mut task.vaddr_pool, memory_poll::get_user_mem_pool(), vaddr, page_cnt,  phy_free);
    unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
}


/**
 * 指定虚拟地址，释放内核空间（需要指定是否释放物理空间）
 */
#[inline(never)]
pub fn free_kernel_page(vaddr: usize, page_cnt: usize, phy_free: bool) {
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().lock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().lock() };
    // 释放内核空间
    memory_deallocation::free_page(memory_poll::get_kernel_addr_pool(), memory_poll::get_kernel_mem_pool(), vaddr, page_cnt,  phy_free);
    unsafe { KERNEL_ADDR_POOL_LOCK.get_mut().unlock() };
    unsafe { KERNEL_MEM_POOL_LOCK.get_mut().unlock() };
}

/**
 * 已知栈顶，分配一个物理页
 */
#[inline(never)]
pub fn malloc_user_page_by_vaddr(vaddr_pool: &mut MemPool, vaddr: usize) {
    if vaddr_pool.is_set(vaddr) {
        MY_PANIC!("addr: 0x{:x} set", vaddr);
    }
    let vaddr_pool_set = vaddr_pool.addr_set(vaddr);
    if !vaddr_pool_set {
        MY_PANIC!("vaddr invalid to pool, could not to set vaddr");
    }
    thread::check_task_stack("failed to malloc user stack memory");
    unsafe { USER_MEM_POOL_LOCK.get_mut().lock() };
    memory_allocation::malloc_phy_by_vaddr(vaddr, memory_poll::get_user_mem_pool());
    unsafe { USER_MEM_POOL_LOCK.get_mut().unlock() };
}


/**
 * 释放某个虚拟地址池的所有内存
 *    - 根据虚拟地址池的开始地址和结束地址
 *    - 遍历页目录表的每一项，然后看页目录项是否赋值
 *    - 如果这个页目录项存在，那么找到其中的页表，然后回收掉这个页表指向的所有内存空间（4MB）以及这个页表自身的空间
 */
#[inline(never)]
pub fn free_by_addr_pool(vaddr_pool: &MemPool) {
    // 遍历虚拟地址池的开始地址
    let start_addr = vaddr_pool.addr_start;
    // 虚拟地址池的结束地址
    let end_addr = start_addr + vaddr_pool.bitmap.bits_len() * vaddr_pool.granularity;

    // 遍历地址池的地址。跨度是4K * 1024（1个页表的地址）
    for vaddr in (start_addr..end_addr).step_by(vaddr_pool.granularity * constants::PAGE_TABLE_ENTRY_COUNT) {
        // 查询页目录项
        let pde = page_util::addr_to_pde(vaddr);
        
        // 如果页目录项都不存在，那么这一项就无需处理了
        if !pde.present() {
            continue;
        }
        // 已知一个虚拟地址，找到这个虚拟地址所在的页表
        let page_table = page_util::page_table(vaddr);
        // 遍历页表，回收掉这个页表指向的所有空间
        for page_entry in page_table.iter() {
            if !page_entry.present() {
                continue;
            }
            // 得到这个页表项中记录的物理地址
            let phy_addr = page_entry.get_phy_addr();
            // 用户内存池释放
            memory_poll::get_user_mem_pool().restore(phy_addr.try_into().unwrap());
        }
        // 释放页表自身（内核内存池）
        memory_poll::get_kernel_mem_pool().restore(pde.get_phy_addr().try_into().unwrap());
    }
}

/**
 * 已知空间page_data，把这块数据复制一份，并且让to_dir_table页目录表可以访问到
 *  @param page_data: 需要复制的数据（当前程序可以访问，经过了当前任务页表的映射）
 *  @param to_dir_table: 数据要拷贝到的页目录表（此时该地址，是当前任务可以访问的虚拟地址），这是其他任务的页目录表
 *  @param page_table: 该数据拷贝到页目录表后，所在的页表（因为可能页表不存在的话就需要申请空间，因此可以为空）（当前任务可访问的虚拟地址）
 *
 */
#[inline(never)]
pub fn copy_single_user_page<'a>(page_data: &[u8], to_dir_table: &mut PageTable, to_page_table: Option<&'a mut PageTable>) -> &'a mut PageTable {
    
    // // 为什么这样就可以？
    // let page_table = if to_page_table.is_some() {
    //     to_page_table.unwrap()
    // } else {
    //     unsafe { &mut *(self::malloc_kernel_page(1) as *mut PageTable) }
    // };
    // return page_table;

    ASSERT!(page_data.len() == constants::PAGE_SIZE as usize);
    let cur_task = &mut thread::current_thread().task_struct;

    /*** 1. 把这一页的数据，先复制一份出来  */
    // 申请一块用户空间，用于存放我们要复制的数据
    let new_page_data_addr = self::malloc_user_page(cur_task, 1);
    let new_page_data = unsafe { core::slice::from_raw_parts_mut(new_page_data_addr as *mut u8, constants::PAGE_SIZE as usize) };
    // 那一页的数据，复制到新的空间中
    new_page_data.copy_from_slice(page_data);


    // /**** 2. 填充页表，页表指向数据页的物理地址  */
    let pde_idx = page_util::locate_pde(page_data.as_ptr() as usize);
    let pte_idx = page_util::locate_pte(page_data.as_ptr() as usize);

    // 页目录项，指向的是页表。看看页目录项有没有值
    // 如果页目录项有值，那么我们是不知道页表的虚拟地址的（只有知道虚拟地址才可以操作），因此需要使用入参的页表地址
    let pde = to_dir_table.get_entry(pde_idx);
    let page_table = if pde.present() {
        ASSERT!(to_page_table.is_some());
        to_page_table.unwrap()
    } else {
        if to_page_table.is_some() {
            self::free_kernel_page(to_page_table.unwrap() as *const _ as usize, 1, false);
        }
        // 如果页目录项没有指向页表，那么需要创建空间给页表
        unsafe { &mut *(self::malloc_kernel_page(1) as *mut PageTable) }
    };

    // 填充页表。页表项指向物理页的物理地址
    page_table.set_entry(pte_idx, PageTableEntry::new_default(page_util::get_phy_from_virtual_addr(new_page_data_addr)));


    /**** 3. 填充页目录表，页目录表该项指向页表的物理地址 *****/
    if !pde.present() {
        // 填充页目录表，页目录项指向 页表
        to_dir_table.set_entry(pde_idx, PageTableEntry::new_default(page_util::get_phy_from_virtual_addr(page_table as *const _ as usize)));
    }

    /**** 4. 释放申请的内存（不要释放物理地址，只是释放该内存空间跟当前任务的链接关系） */
    self::free_user_page(cur_task, new_page_data_addr, 1, false);

    page_table
}
