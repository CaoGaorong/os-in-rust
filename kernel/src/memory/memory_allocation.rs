
use core::{mem::size_of, ptr};

use os_in_rust_common::{constants, pool::MemPool, printk, printkln, utils, ASSERT, MY_PANIC};

use crate::{page_util, thread};

use super::{mem_block::{self, Arena, MemBlockAllocator}, memory_poll::{self, get_kernel_addr_pool, get_kernel_mem_pool, get_user_mem_pool}};

/**
 * ************************************************************
 * *       本文件是针对内存的分配操作              
 * ************************************************************
 */

/**
 * 申请内存
 */
pub fn malloc<T>(bytes: usize) -> &'static mut T {
    let addr = sys_malloc(bytes);
    unsafe { &mut *(addr as *mut T) }
}

/**
 * 在内核空间申请bytes字节的空间
 */
#[inline(never)]
pub fn sys_malloc(bytes: usize) -> usize {
    // 当前任务
    let task = &mut thread::current_thread().task_struct;

    // 找出物理内存池。内核程序或者用户程序
    if task.pgdir == ptr::null_mut() {
        malloc_bytes(&mut get_kernel_addr_pool().lock(), &mut get_kernel_mem_pool().lock() , mem_block::get_kernel_mem_block_allocator(), bytes)
    } else {
        malloc_bytes(&mut task.vaddr_pool, &mut get_user_mem_pool().lock(), &mut task.mem_block_allocator, bytes)
    }
}

/**
 * 在某个Task中，从堆内存中分配bytes个字节
 */
fn malloc_bytes(vaddr_pool: &mut MemPool, phy_mem_pool: &mut MemPool, allocator: &'static mut MemBlockAllocator, bytes: usize) -> usize {
    // 如果申请很大量的字节空间，直接分配整页
    if bytes > constants::MINIMAL_BLOCK_SIZE * 2usize.pow(constants::MEM_BLOCK_CONTAINER_CNT as u32 - 1) {
        // 计算需要申请多少个页
        let pages = utils::div_ceil((size_of::<Arena>() + bytes) as u32, constants::PAGE_SIZE) as usize;
        // 开始申请页
        let page_addr = malloc_page(vaddr_pool, phy_mem_pool, pages as usize);
        // 申请到的页，转成一个Arena
        let arena = unsafe { &mut *(page_addr as *mut Arena) };
        // 初始化arena
        arena.init(ptr::null_mut(), pages, (pages * constants::PAGE_SIZE as usize - size_of::<Arena>()));
        // arena首个mem_block的地址
        return arena.find_mem_block(0) as *const _ as usize;
    }

    // 先根据要申请的字节数量，找到匹配的容器
    let container = allocator.match_container(bytes);
    let addr = container as *const _ as usize;
    // printkln!("container addr: 0x{:x}", addr);
    
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
        container.lock.unlock();
        return mem_block as *const _ as usize;
    }

    // 如果已经没有可用的块了，那么需要申请1页
    let page_addr = malloc_page(vaddr_pool, phy_mem_pool, 1);
    // 申请到的页，转成一个Arena
    let arena = unsafe { &mut *(page_addr as *mut Arena) };
    // 把这一页的物理空间，初始化成arena
    arena.init(container as *mut _, 1, container.block_size());
    
    // 把这一个新鲜出炉的Arena，剁碎了，放入到容器中
    container.smash(arena);


    // 从容器申请一个
    let first_block = container.apply().unwrap();
    // 申请了1个
    arena.apply_one();
    
    container.lock.unlock();
    return first_block as *const _ as usize;
}

/**
 * 申请page_cnt个内核页。得到虚拟地址
 */
pub fn malloc_kernel_page(page_cnt: usize) -> usize{ 
    // println!("malloc kernel  {} page", page_cnt);
    malloc_page(&mut get_kernel_addr_pool().lock(), &mut get_kernel_mem_pool().lock(), page_cnt)
}


/**
 * 从addr_pool地址池中申请连续的page_cnt页虚拟地址，从mem_pool中申请不连续的page_cnt物理页，并且构建虚拟地址和物理地址的页表联系。返回虚拟起始地址
 */
fn malloc_page(addr_pool: &mut MemPool, mem_pool: &mut MemPool, page_cnt: usize) -> usize {
    // 从虚拟地址池中申请连续的虚拟地址
    let addr_apply_res = addr_pool.apply(page_cnt);
    if addr_apply_res.is_err() {
        MY_PANIC!("failed to apply virtual address.  res: {:?}", addr_apply_res);
    }
    
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
    malloc_phy_by_vaddr(user_stack_top, &mut get_user_mem_pool().lock());
}