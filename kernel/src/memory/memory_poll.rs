use core::{mem::{self}, slice};

use os_in_rust_common::{constants, paging::PageTable, pool::MemPool, racy_cell::RacyCell};


/**
 * ************************************************************
 * *       本文件是针对全局内存池的操作
 * ************************************************************
 */


/**
 * 内核物理内存池
 */
static KERNEL_MEM_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());

/**
 * 用户物理内存池
 */
static USER_MEM_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());

/**
 * 用户地址池
 */
static KERNEL_ADDR_POOL: RacyCell<MemPool> = RacyCell::new(MemPool::empty());


/**
 * 获取内核物理内存池
 */
pub fn get_kernel_mem_pool() -> &'static mut MemPool {
    unsafe { KERNEL_MEM_POOL.get_mut() }
}

/**
 * 获取用户物理内存池
 */
pub fn get_user_mem_pool() -> &'static mut MemPool {
    unsafe { USER_MEM_POOL.get_mut() }
}

/**
 * 获取内核虚拟地址内存池
 */
pub fn get_kernel_addr_pool() -> &'static mut MemPool {
    unsafe { KERNEL_ADDR_POOL.get_mut() }
}

/**
 * 初始化内存池
 * all_mem: 总物理内存
 * 总物理内存减去已经使用的，可以得到可用的物理内存。
 *
 * 初始化内存池，把可用物理内存分为两个部分：
 *      - 内核内存
 *      - 用户内存
 */
#[inline(never)]
pub fn mem_pool_init(all_mem: u32) {
    // 目前已经用了的内存空间：低端1MB + 内核页目录表（1个） + 内核页表（255个）
    let used_mem = constants::REAL_MEMORY
        + mem::size_of::<PageTable>()
        + mem::size_of::<PageTable>() * constants::PAGE_TABLE_ENTRY_COUNT;

    // 可用的内存 = 总内存 - 已用内存
    let available_mem_size = all_mem - used_mem as u32;

    // 把可用的内存，换算成页。不足一页的部分丢弃
    let available_page = available_mem_size / constants::PAGE_SIZE;

    // 每个池子，要映射的页数
    let pages_for_pool = available_page / 2;

    // 位图自身所在的地址自身地址
    let mut bit_map_addr = constants::KERNEL_MEM_BITMAP_ADDR as usize;


    /* 1. 内核物理内存池 */
    let kernel_mem_pool = unsafe { KERNEL_MEM_POOL.get_mut() };
    *kernel_mem_pool = compose_pool(
        // 池子描述的内存起始地址。也就是可用内存之上
        used_mem, 
        // 该可以描述多少页内存空间
        pages_for_pool, 
        // 该池子位图自身所在的地址
        bit_map_addr
    );

    // 用户物理内存池位图自身地址
    bit_map_addr += kernel_mem_pool.bitmap.size;

    /* 2. 用户物理内存池 */
    let user_mem_pool = unsafe { USER_MEM_POOL.get_mut() };
    *user_mem_pool = compose_pool(
        // 池子描述的内存起始地址
        used_mem + (pages_for_pool * constants::PAGE_SIZE) as usize, 
        pages_for_pool, 
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
        pages_for_pool, 
        // 这个池子的位图，位于上面两个池子的位图之上
        bit_map_addr
    );

    // println!("kernel_mem_pool  addr_start: 0x{:x}", kernel_mem_pool.addr_start as u32);
    // println!("kernel_mem_pool  bitmap addr: 0x{:x}", kernel_mem_pool.bitmap.map_ptr as u32);
    // println!("kernel_mem_pool  bitmap len: 0x{:x}", kernel_mem_pool.bitmap.size as u32);


    // println!("user_mem_pool  addr_start: 0x{:x}", user_mem_pool.addr_start as u32);
    // println!("user_mem_pool  bitmap addr: 0x{:x}", user_mem_pool.bitmap.map_ptr as u32);
    // println!("user_mem_pool  bitmap len: 0x{:x}", user_mem_pool.bitmap.size as u32);

    // println!("kernel_addr_pool  addr_start: 0x{:x}", kernel_addr_pool.addr_start  as u32);
    // println!("kernel_addr_pool  bitmap addr: 0x{:x}", kernel_addr_pool.bitmap.map_ptr as u32);
    // println!("kernel_addr_pool  bitmap len: 0x{:x}", kernel_addr_pool.bitmap.size as u32);



}

/**
 * 构建一个内存池
 * addr_start: 该内存池描述的起始地址
 * mem_page_num: 该内存池描述的内存块页数量
 * bitmap_base_addr: 该内存池位图自身所在的内存地址
 */
#[inline(never)]
fn compose_pool(addr_start: usize, mem_page_num: u32, bitmap_base_addr: usize) -> MemPool {
    let mut mem_pool = MemPool::empty();
    
    // 构建bitmap
    let bitmap: &mut [u8] = unsafe {
        slice::from_raw_parts_mut(
            // bitmap自身所在的地址
            bitmap_base_addr as *mut _,
            // bitmap需要的长度。
            (mem_page_num / 8).try_into().unwrap(),
        )
    };
    mem_pool.init(addr_start, constants::PAGE_SIZE as usize, bitmap);
    mem_pool
}