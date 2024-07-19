use core::mem::{self, size_of};

use lazy_static::lazy_static;
use os_in_rust_common::{constants, elem2entry, linked_list::{LinkedList, LinkedNode}, printk, printkln, racy_cell::RacyCell, vga::print, ASSERT, MY_PANIC};
use crate::{println, thread};

use crate::sync::Lock;

/**
 * 这里是对内存块进行管理
 */

/**
 * 内核全局内存块分配器（所有内核线程共用一个页表和内存池，因此所有内核线程用的内存块容器也是同一个）
 */
lazy_static! {
    static ref KERNEL_MEMORY_BLOCK_ALLOCATOR: RacyCell<MemBlockAllocator> = RacyCell::new(MemBlockAllocator::new());
}

/**
 * 获取内核内存块分配器
 */
pub fn get_kernel_mem_block_allocator() -> &'static mut MemBlockAllocator {
    unsafe { KERNEL_MEMORY_BLOCK_ALLOCATOR.get_mut() }
}

/**
 * 内存块分配器。包含多种规格的内存块容器
 */
pub struct MemBlockAllocator {
    /**
     * 内存块的容器。包含多种规格的容器
     */
    containers: [MemBlockContainer; constants::MEM_BLOCK_CONTAINER_CNT],
}

impl MemBlockAllocator {
    pub fn new() -> MemBlockAllocator {
        // 构建一个数组
        const EMPTY_VALUE: MemBlockContainer = MemBlockContainer::empty();
        let mut container_list = [EMPTY_VALUE; constants::MEM_BLOCK_CONTAINER_CNT];
        // 初始化
        let mut block_size = constants::MINIMAL_BLOCK_SIZE;
        for idx in 0 .. container_list.len() {
            container_list[idx].block_size = block_size;
            block_size *= 2;
        }
        Self {
            containers: container_list,
        }
    }

    /**
     * 根据要分配的空间大小，找出大小匹配的容器
     */
    #[inline(never)]
    pub fn match_container(&'static mut self, bytes: usize) -> &'static mut MemBlockContainer {
        let containers = &mut self.containers;
        for container in containers.iter_mut() {
            if bytes <= container.block_size {
                return container;
            }
        }
        MY_PANIC!("SYSTEM ERROR, No mem size match. mem_size: {}", bytes);
        panic!("error")
    }

    pub fn print_container_size(&self) {
        self.containers.iter().for_each(|e| {
            printk!("{}:{} ", e.block_size, e.size());
        });
        printkln!();
    }
    pub fn print_container(&self) {
        // printk!("mem block container size: ");
        // for ele in self.containers.iter() {
        //     printk!("{} ", ele.block_size);
        // }
        // printkln!();

        self.containers.iter().for_each(|e| {
            printkln!("mem block size: {}, left count:{}", e.block_size, e.available_blocks_list.iter().count())
        });
    }
}



/**
 * 内存块的容器（只保存某种特定规格的内存块）
 * 一个容器包含多个相同大小的内存块（使用链表相连）
 */
pub struct MemBlockContainer {
    /**
     * 容器加锁
     */
    pub lock: Lock,

    /**
     * 该容器内每个内存块的大小
     */
    block_size: usize,
    /**
     * 可用的内存块列表
     */
    pub available_blocks_list: LinkedList,
}
impl MemBlockContainer {
    pub const fn empty() -> Self {
        Self {
            lock: Lock::new(),
            block_size: 0,
            available_blocks_list: LinkedList::new(),
        }
    }

    /**
     * 从容器里面申请1个内存块
     */
    #[no_mangle]
    #[inline(never)]
    pub fn apply(&mut self) -> Option<&'static mut MemBlock>{
        // 终于找到一个合适的规格。从里面空余链表找一个
        let block_list = &mut self.available_blocks_list;
        // 但是没有可用的内存块了
        if block_list.is_empty() {
            return Option::None;
        }
        // 从可用内存块列表中，取出一个内存块
        let mem_block_tag = block_list.pop();
        let mem_block = MemBlock::parse_by_tag(mem_block_tag);
        unsafe { (mem_block as *mut _ as *mut u8).write_bytes(0, mem_block.arena_addr().block_size) };
        return Option::Some(mem_block);
    }

    /**
     * 把某一个内存块空间，释放回容器中
     */
    pub fn release(&mut self, mem_block: &mut MemBlock) {
        // mem_block.init();
        // 把这块内存空间放回容器
        self.available_blocks_list.append(&mut mem_block.tag);
    }

    /**
     * 把某一大块空间“剁碎”了放入到容器中
     */
    pub fn smash(&mut self, arena: &Arena) {
        // 首先确保arena没有被用过
        ASSERT!(!arena.in_use());

        // 这块空间可以碎成blocks块
        // 遍历每一块
        for block_idx in 0 .. arena.left_blocks() {
            // 找到这一个mem_block
            let mem_block = arena.find_mem_block(block_idx);
            // 放到可用列表里面
            self.available_blocks_list.append(&mut mem_block.tag);
        }
    }

    /**
     * 把某一个mem_block从容器中移除
     */
    pub fn remove_mem_block(&mut self, mem_block: &MemBlock) {
        self.available_blocks_list.remove(&mem_block.tag);
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn size(&self) -> usize {
        self.available_blocks_list.size()
    }
}

/**
 * 单个内存块。
 *  当没有被分配的时候，通过标签可以串起来
 */
pub struct MemBlock {
    /**
     * 每个内存块，包含一个标签
     * 这样内存块可以串起来
     */
    tag: LinkedNode,
}
impl MemBlock {
    pub const fn new() -> Self {
        Self {
            tag: LinkedNode::new(),
        }
    }
    /**
     * 初始化
     */
    pub const fn init(&mut self) {
        self.tag = LinkedNode::new();
    }
    /**
     * 通过tag的偏移量，得到外面的内存块的地址
     */
    pub fn parse_by_tag(tag_offset: &LinkedNode) -> &'static mut MemBlock {
        let addr = elem2entry!(MemBlock, tag, tag_offset as *const _ as usize);
        unsafe { &mut *addr }
    }

    /**
     * 根据当前mem_block，找到所在的Arena
     */
    pub fn arena_addr(&self) -> &'static mut Arena {
        unsafe { &mut *(((self as *const _ as u32) & 0xfffff000) as *mut Arena) }
    }
}

/**
 * Arena是一大块空间（x物理页），包括两个部分：
 *   - Arena 元数据（本结构体）
 *   - mem_block
 */
pub struct Arena {
    /**
     * 该arena为哪个容器提供的内存块
     */
    supply_for: *mut MemBlockContainer,
    /**
     * 该arena占用物理页的数量。回收空间的时候按照整页回收
     */
    occupy_page_cnt: usize,
    /**
     * 该arena内每个内存块的大小
     */
    block_size: usize,
    /**
     * 该arena未使用的内存块数量
     */
    left_block_cnt: usize,
}
impl Arena {
    pub fn init(&mut self, supply_for: *mut MemBlockContainer, occupy_page_cnt: usize, block_size: usize) {
        // 把内存块数据清零
        unsafe { (self as *mut Self as *mut u8).write_bytes(0, occupy_page_cnt * constants::PAGE_SIZE as usize) };
        self.supply_for = supply_for;
        self.block_size = block_size;
        self.occupy_page_cnt = occupy_page_cnt;
        self.left_block_cnt = (occupy_page_cnt * constants::PAGE_SIZE as usize - size_of::<Arena>()) / block_size;
    }
    /**
     * 根据内存块的下标，在当前Arena中找到这个内存块
     */
    pub fn find_mem_block(&self, block_idx: usize) -> &'static mut MemBlock {
        let mem_addr = (self as *const _ as usize) + size_of::<Arena>() + block_idx * self.block_size;
        unsafe { &mut *(mem_addr as *mut MemBlock) }
    }
    
    pub fn apply_one(&mut self) {
        self.left_block_cnt -= 1;
    }

    pub fn release_one(&mut self) {
        self.left_block_cnt += 1;
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn left_blocks(&self) -> usize {
        self.left_block_cnt
    }
    pub fn occupy_pages(&self) -> usize {
        self.occupy_page_cnt
    }

    pub fn supply_for(&self) ->  *mut MemBlockContainer {
        self.supply_for
    }

    /**
     * 这个Arena是否在使用中
     */
    pub fn in_use(&self) -> bool {
        // 总可用的mem_block = (总空间 - arena头) / 单个mem_block大小
        let total_blocks = (self.occupy_page_cnt * constants::PAGE_SIZE as usize - size_of::<Arena>()) / self.block_size;
        // 如果目前剩余的mem_block和总mem_block不相等，说明有mem_block被使用了
        self.left_block_cnt != total_blocks
    }
    
}