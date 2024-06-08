use core::{mem::{self, size_of}, ops::DerefMut, ptr::{self, null_mut}, slice};

use os_in_rust_common::{bitmap::MemoryError, constants, paging::{self, PageTable}, pool::MemPool, printkln, racy_cell::RacyCell, utils, ASSERT};

use crate::{mem_block::{self, Arena, MemBlockAllocator}, mutex::Mutex, page_util, thread::{self}};

/**
 * 内核物理内存池
 */
pub static KERNEL_MEM_POOL: RacyCell<Mutex<MemPool>> = RacyCell::new(Mutex::new(MemPool::empty()));

/**
 * 用户物理内存池
 */
pub static USER_MEM_POOL: RacyCell<Mutex<MemPool>> = RacyCell::new(Mutex::new(MemPool::empty()));

/**
 * 用户地址池
 */
static KERNEL_ADDR_POOL: RacyCell<Mutex<MemPool>> = RacyCell::new(Mutex::new(MemPool::empty()));

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
    let mut kernel_mem_pool = unsafe { KERNEL_MEM_POOL.get_mut().lock() };
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
    let mut user_mem_pool = unsafe { USER_MEM_POOL.get_mut().lock() };
    *user_mem_pool = compose_pool(
        // 池子描述的内存起始地址
        used_mem + (pages_for_pool * constants::PAGE_SIZE) as usize, 
        pages_for_pool, 
        bit_map_addr
    );
    
    // 内核虚拟地址池位图自身地址
    bit_map_addr += user_mem_pool.bitmap.size;

    /* 3. 内核虚拟地址池 */
    let mut kernel_addr_pool = unsafe { KERNEL_ADDR_POOL.get_mut().lock() };
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


/**
 * 在内核空间申请bytes字节的空间
 */
pub fn malloc(bytes: usize) -> usize {
    // 当前任务
    let task = &mut thread::current_thread().task_struct;

    // 找出物理内存池。内核程序或者用户程序
    if task.pgdir == null_mut() {
        malloc_bytes(unsafe { &mut KERNEL_ADDR_POOL.get_mut().lock()}, unsafe { &mut KERNEL_MEM_POOL.get_mut().lock() }, mem_block::get_kernel_mem_block_allocator(), bytes)
    } else {
        malloc_bytes(&mut task.vaddr_pool, unsafe { &mut USER_MEM_POOL.get_mut().lock() }, &mut task.mem_block_allocator, bytes)
    }
}

/**
 * 在某个Task中，从堆内存中分配bytes个字节
 */
pub fn malloc_bytes(vaddr_pool: &mut MemPool, phy_mem_pool: &mut MemPool, allocator: &'static mut MemBlockAllocator, bytes: usize) -> usize {
    // 如果申请很大量的字节空间，直接分配整页
    if bytes > 1024 {
        // 计算需要申请多少个页
        let pages = utils::div_ceil((size_of::<Arena>() + bytes) as u32, constants::PAGE_SIZE) as usize;
        // 开始申请页
        let page_addr = malloc_page(vaddr_pool, phy_mem_pool, pages as usize);
        // 申请到的页，转成一个Arena
        let arena = unsafe { &mut *(page_addr as *mut Arena) };
        // 初始化arena
        arena.init(ptr::null_mut(), pages * constants::PAGE_SIZE as usize - size_of::<Arena>(), 0);
        // 申请到的空间，跳过arena的大小。那么就是可用mem_block的地址
        return page_addr + size_of::<Arena>();
    }

    // 先根据要申请的字节数量，找到匹配的容器
    let container = allocator.match_container(bytes);
    
    // 上锁
    container.lock.lock();

    // 从这个容器中申请一个内存块
    let mem_block_apply = container.apply();

    // 如果有可用的内存块
    if mem_block_apply.is_some() {
        // 申请到的内存块地址
        let mem_block = mem_block_apply.unwrap();
        // arena - 1
        let arena  = mem_block.arena_addr();
        arena.apply_one();
        return mem_block as *const _ as usize;
    }

    // 如果已经没有可用的块了，那么需要申请1页
    let page_addr = malloc_page(vaddr_pool, phy_mem_pool, 1);
    // 申请到的页，转成一个Arena
    let arena = unsafe { &mut *(page_addr as *mut Arena) };

    // 内存块的数量
    let blocks_cnt = (constants::PAGE_SIZE as usize - size_of::<Arena>())  / container.block_size();
    // 初始化arena
    arena.init(container as *mut _, container.block_size(), blocks_cnt);
    arena.apply_one();
    // arena中第一个可用内存块的地址
    let first_block_addr = page_addr + size_of::<Arena>();

    // 把申请到的这一页空间剩余的空间，剁碎了，放入到容器中
    container.smash(first_block_addr + arena.block_size, arena.left_block_cnt);
    container.lock.unlock();
    return first_block_addr;
}

/**
 * 申请page_cnt个内核页。得到虚拟地址
 */
pub fn malloc_kernel_page(page_cnt: usize) -> usize{ 
    // println!("malloc kernel  {} page", page_cnt);
    malloc_page(unsafe { &mut KERNEL_ADDR_POOL.get_mut().lock()}, unsafe { &mut KERNEL_MEM_POOL.get_mut().lock() }, page_cnt)
}

/**
 * 在user_addr_pool虚拟地址池中，申请page_cnt个用户页。得到虚拟地址
 */
pub fn malloc_user_page(user_addr_pool: &mut MemPool, page_cnt: usize) -> usize{
    malloc_page(user_addr_pool, unsafe { &mut USER_MEM_POOL.get_mut().lock() }, page_cnt)
}


/**
 * 从addr_pool地址池中申请连续的page_cnt页虚拟地址，从mem_pool中申请不连续的page_cnt物理页，并且构建虚拟地址和物理地址的页表联系。返回虚拟起始地址
 */
fn malloc_page(addr_pool: &mut MemPool, mem_pool: &mut MemPool, page_cnt: usize) -> usize {
    // 从虚拟地址池中申请连续的虚拟地址
    let addr_apply_res = addr_pool.apply(page_cnt);
    ASSERT!(addr_apply_res.is_ok());
    
    let base_virtual_addr  = addr_apply_res.unwrap();

    let mut virtual_addr = base_virtual_addr;
    for _ in 0..page_cnt {
        // 给定虚拟地址，申请一个物理空间，并且建立虚拟地址和该物理空间的联系
        malloc_phy_by_vaddr(virtual_addr, mem_pool);

        virtual_addr += constants::PAGE_SIZE as usize;
    }
    base_virtual_addr
}

/**
 * 已知虚拟地址virtual_addr，然后前往mem_pool物理空间池申请1页空间，并且返回物理空间池的物理地址
 */
fn malloc_phy_by_vaddr(virtual_addr: usize, mem_pool: &mut MemPool) -> usize{
    let mem_apply_res = mem_pool.apply_one();
    ASSERT!(mem_apply_res.is_ok());
    // 从物理地址池中申请1页
    let phy_addr = mem_apply_res.unwrap();

    // 构建页表，把两者连起来
    page_util::add_page_connection(virtual_addr, phy_addr);

    phy_addr
}

/**
 * 已知栈顶，分配一个物理页
 */
pub fn malloc_user_stack_page(user_stack_top: usize) {
    malloc_phy_by_vaddr(user_stack_top, unsafe { &mut USER_MEM_POOL.get_mut().lock()});
}