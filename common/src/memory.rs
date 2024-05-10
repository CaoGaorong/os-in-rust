use core::{mem, slice};

use crate::racy_cell::RacyCell;
use crate::{constants, pool::MemPool};
use crate::{page_util, paging, println};

/**
 * 内核物理内存池
 */
pub static KERNEL_MEM_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());

/**
 * 用户物理内存池
 */
pub static USER_MEM_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());

/**
 * 用户地址池
 */
static KERNEL_ADDR_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());

/**
 * 初始化内存池
 * all_mem: 总物理内存
 * 总物理内存减去已经使用的，可以得到可用的物理内存。
 *
 * 初始化内存池，把可用物理内存分为两个部分：
 *      - 内核内存
 *      - 用户内存
 */
pub fn mem_pool_init(all_mem: u32) {
    // 目前已经用了的内存空间：低端1MB + 内核页目录表（1个） + 内核页表（255个）
    let used_mem = constants::REAL_MEMORY
        + mem::size_of_val(paging::get_dir_ref())
        + mem::size_of_val(paging::get_table_list_ref());

    // 可用的内存 = 总内存 - 已用内存
    let available_mem_size = all_mem - used_mem as u32;

    // 把可用的内存，换算成页。不足一页的部分丢弃
    let available_page = available_mem_size / constants::PAGE_SIZE;

    // 每个池子，要映射的页数
    let page_for_pool = available_page / 2;

    // 位图自身所在的地址自身地址
    let mut bit_map_addr = constants::KERNEL_MEM_BITMAP_ADDR as usize;


    /* 1. 内核物理内存池 */
    let kernel_mem_pool = unsafe { KERNEL_MEM_POOL.get_mut() };
    *kernel_mem_pool = compose_pool(
        // 池子描述的内存起始地址。也就是可用内存之上
        used_mem, 
        // 该可以描述多少页内存空间
        page_for_pool, 
        // 该池子位图自身所在的地址
        bit_map_addr
    );

    // 用户物理内存池位图自身地址
    bit_map_addr += kernel_mem_pool.bitmap.size;

    /* 2. 用户物理内存池 */
    let user_mem_pool = unsafe { USER_MEM_POOL.get_mut() };
    *user_mem_pool = compose_pool(
        // 池子描述的内存起始地址
        used_mem + (page_for_pool * constants::PAGE_SIZE) as usize, 
        page_for_pool, 
        bit_map_addr
    );
    
    // 内核虚拟地址池位图自身地址
    bit_map_addr += user_mem_pool.bitmap.size;

    /* 3. 内核虚拟地址池 */
    let kernel_addr_pool = unsafe { KERNEL_ADDR_POOL.get_mut() };
    *kernel_addr_pool = compose_pool(
        // 虚拟地址的开始。位于高端1G，再跨过1MB
        constants::KERNEL_ADDR_START + constants::REAL_MEMORY, 
        // 这个池子的大小。虚拟地址跟物理地址大小一致
        page_for_pool, 
        // 这个池子的位图，位于上面两个池子的位图之上
        bit_map_addr
    );

    // println!("kernel_mem_pool  addr_start: 0x{:x}", kernel_mem_pool.addr_start);
    // println!("kernel_mem_pool  bitmap addr: 0x{:x}", kernel_mem_pool.bitmap.map_ptr as u32);
    // println!("kernel_mem_pool  bitmap len: 0x{:x}", kernel_mem_pool.bitmap.size as u32);


    // println!("user_mem_pool  addr_start: 0x{:x}", user_mem_pool.addr_start);
    // println!("user_mem_pool  bitmap addr: 0x{:x}", user_mem_pool.bitmap.map_ptr as u32);
    // println!("user_mem_pool  bitmap len: 0x{:x}", user_mem_pool.bitmap.size as u32);

    // println!("kernel_addr_pool  addr_start: 0x{:x}", kernel_addr_pool.addr_start);
    // println!("kernel_addr_pool  bitmap addr: 0x{:x}", kernel_addr_pool.bitmap.map_ptr as u32);
    // println!("kernel_addr_pool  bitmap len: 0x{:x}", kernel_addr_pool.bitmap.size as u32);



}

/**
 * 构建一个内存池
 * addr_start: 该内存池描述的起始地址
 * pool_page: 该内存池描述的内存块页大小
 * bitmap_base_addr: 该内存池位图自身所在的内存地址
 */
fn compose_pool(addr_start: usize, pool_page: u32, bitmap_base_addr: usize) -> MemPool {
    let mut mem_pool = MemPool::empty();
    
    // 构建bitmap
    let bitmap: &mut [u8] = unsafe {
        slice::from_raw_parts_mut(
            // bitmap自身所在的地址
            bitmap_base_addr as *mut _,
            // bitmap需要的长度。
            (pool_page / 8).try_into().unwrap(),
        )
    };
    mem_pool.init(addr_start, constants::PAGE_SIZE as usize, bitmap);
    mem_pool
}

/**
 * 申请page_cnt个内核页。得到虚拟地址
 */
pub fn malloc_kernel_page(page_cnt: usize) -> u32{ 
    // 申请连续的虚拟地址
    let base_virtual_addr  = unsafe { KERNEL_ADDR_POOL.get_mut().apply(page_cnt).unwrap() };

    let mut virtual_addr = base_virtual_addr;
    for _ in 0..page_cnt {
        // 得到该页的物理地址
        let phy_addr = unsafe { KERNEL_MEM_POOL.get_mut().apply_one().unwrap() };

        // 构建页表，把两者连起来
        page_util::add_page_connection(virtual_addr, phy_addr);

        virtual_addr += constants::PAGE_SIZE;
    }
    base_virtual_addr
}
