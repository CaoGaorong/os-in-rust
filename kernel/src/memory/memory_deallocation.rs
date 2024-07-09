

use core::ptr;

use os_in_rust_common::{constants, pool::MemPool, printk, printkln, ASSERT};

use crate::{page_util, thread};

use super::{mem_block::{Arena, MemBlock, MemBlockAllocator}, memory_poll::{get_kernel_addr_pool, get_kernel_mem_pool, get_user_mem_pool}};



/**
 * ************************************************************
 * *       本文件是针对内存的释放操作              
 * ************************************************************
 */

/**
 * 是否某一块空间
 *  - vaddr_to_free: 要释放的空间的地址
 */
pub fn sys_free(vaddr_to_free: usize) {
    // 当前任务
    let task = &mut thread::current_thread().task_struct;
    // 找出物理内存池。内核程序或者用户程序
    if task.pgdir == ptr::null_mut() {
        free_bytes(&mut get_kernel_addr_pool().lock(), &mut get_kernel_mem_pool().lock() , vaddr_to_free);
    } else {
        free_bytes(&mut task.vaddr_pool, &mut get_user_mem_pool().lock(), vaddr_to_free);
    }
}

/**
 * 释放字节空间
 *   - addr_pool：地址池
 *   - mem_pool：物理空间内存池
 *   - vaddr_to_free：要释放的地址（注意这个是mem_block的地址，而不是arena的地址）
 */
fn free_bytes(addr_pool: &mut MemPool, mem_pool: &mut MemPool, vaddr_to_free: usize) {
    // 根据要释放的那个地址，转成mem_block
    let mem_block = unsafe { &mut *(vaddr_to_free as *mut MemBlock) };
    // 然后找到该内存块归属的arena
    let arena = mem_block.arena_addr();
    // 对于大页，没有经过容器，所以直接释放掉
    if !arena.in_use() && arena.supply_for() == ptr::null_mut() {
        // 释放整页。把该arena占用的内存页直接释放
        free_page(addr_pool, mem_pool, arena as *const _ as usize, arena.occupy_pages());
        return;
    }
    // 如果arena整整齐齐了，那么可以释放整个页了

    // 1. 根据根据arena的内存块所述的容器
    let container = unsafe { &mut *arena.supply_for() };
    // 上锁
    container.lock.lock();

    // 2.1 把内存块放回容器中
    container.release(mem_block);
    // 2.2 arena中也多一个可用的空间
    arena.release_one();

    // 如果arena还在使用，那么没有 后续操作了
    if arena.in_use() {
        container.lock.unlock();
        return;
    }

    // 3. 把arena所有的内存块，从容器中移除
    for block_idx in 0 .. arena.left_blocks() {
        // 找到内存块
        let arena_block = arena.find_mem_block(block_idx);
        // 从容器移除
        container.remove_mem_block(arena_block);
    }
    
    
    // 4. 释放Arena所占的空间
    free_page(addr_pool, mem_pool, arena as *const _ as usize, arena.occupy_pages());
    container.lock.unlock();
}

/**
 * 释放页空间
 * - addr_pool: 释放空间的虚拟地址池
 * - mem_pool: 释放空间的物理地址池
 * - vaddr_start: 要释放的虚拟起始地址
 * - page_cnt: 要释放的页的数量
 */
fn free_page(addr_pool: &mut MemPool, mem_pool: &mut MemPool, vaddr_start: usize, page_cnt: usize) {
    // 确保这个释放的地址，在当前的虚拟地址池中
    ASSERT!(addr_pool.in_pool(vaddr_start));
    ASSERT!(addr_pool.in_pool(vaddr_start + constants::PAGE_SIZE as usize * page_cnt));
    ASSERT!(page_cnt >= 1 && vaddr_start % constants::PAGE_SIZE as usize == 0);

    // 确保释放的物理地址，也在物理地址池中
    let phy_addr_start = page_util::get_phy_from_virtual_addr(vaddr_start);
    ASSERT!(mem_pool.in_pool(phy_addr_start));
    ASSERT!(phy_addr_start % constants::PAGE_SIZE as usize == 0);

    // 遍历每一页
    for page_idx in 0 .. page_cnt {
        // 虚拟地址
        let vaddr = vaddr_start + page_idx * constants::PAGE_SIZE as usize;
        ASSERT!(addr_pool.in_pool(vaddr));

        // 物理地址
        let phy_addr = page_util::get_phy_from_virtual_addr(vaddr);
        ASSERT!(mem_pool.in_pool(phy_addr));

        // 把物理地址放回池子中
        mem_pool.restore(phy_addr);

        // 取消该虚拟地址页表项的p位
        page_util::unset_pte(vaddr);

        // 把虚拟地址也放回池子中
        addr_pool.restore(vaddr);
    }
}
