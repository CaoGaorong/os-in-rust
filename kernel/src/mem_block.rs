use core::mem::{self, size_of};

use lazy_static::lazy_static;
use os_in_rust_common::{constants, elem2entry, linked_list::{LinkedList, LinkedNode}, printk, printkln, racy_cell::RacyCell, vga::print, ASSERT, MY_PANIC};

use crate::{mem_block, mutex::Mutex, println, sync::Lock};

/**
 * 这里是对内存块进行管理
 */

/**
 * 内核全局内存块分配器（所有内核线程共用一个页表和内存池，因此所有内核线程用的内存块容器也是同一个）
 */
lazy_static! {
    static ref KERNEL_MEMORY_BLOCK_ALLOCATOR: RacyCell<MemBlockAllocator> = RacyCell::new(MemBlockAllocator::new());
}


pub fn get_kernel_mem_block_allocator() -> &'static mut MemBlockAllocator {
    unsafe { KERNEL_MEMORY_BLOCK_ALLOCATOR.get_mut() }
}

// #[repr(C, packed)]
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
    pub fn match_container(&'static mut self, bytes: usize) -> &'static mut MemBlockContainer {
        let container = &mut self.containers;
        for container in container.iter_mut() {
            if bytes <= container.block_size {
                return container;
            }
        }
        MY_PANIC!("SYSTEM ERROR, No mem size match. mem_size: ");
        panic!("error")
    }

    pub fn print_container(&self) {
        printk!("mem block container size: ");
        for ele in self.containers.iter() {
            printk!("{} ", ele.block_size);
        }
        printkln!();

        self.containers.iter().for_each(|e| {
            printkln!("mem block size: {}, left count:{}", e.block_size, e.available_blocks_list.iter().count())
        });
    }
}



/**
 * 内存块的容器（只保存某种特定规格的内存块）
 */

// #[repr(C, packed)]
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
    available_blocks_list: LinkedList,
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
        return Option::Some(mem_block);
    }

    /**
     * 把某一个内存块空间，释放回容器中
     */
    pub fn release(&mut self, vaddr: u32) {
        // 把这块内存空间放回容器
        let mem_block = unsafe { &mut *(vaddr as *mut MemBlock) };
        self.available_blocks_list.append(&mut mem_block.tag);
    }

    /**
     * 把某一大块空间“剁碎”了放入到容器中
     */
    pub fn smash(&mut self, first_block_addr: usize, block_cnt: usize) {
        // 这块空间可以碎成blocks块
        // 遍历每一块
        for block_idx in 0 .. block_cnt {
            // 这一块的地址
            let block_addr = first_block_addr + block_idx * self.block_size;
            // 转成内存块
            let mem_block = unsafe { &mut *(block_addr as *mut MemBlock) };
            // 放到可用列表里面
            self.available_blocks_list.append(&mut mem_block.tag);
        }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }
}

#[repr(C)]
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
     * 通过tag的偏移量，得到外面的内存块的地址
     */
    pub fn parse_by_tag(tag_offset: &LinkedNode) -> &'static mut MemBlock {
        let addr = elem2entry!(MemBlock, tag, tag_offset as *const _ as usize);
        unsafe { &mut *addr }
    }

    /**
     * 
     */
    pub fn arena_addr(& self) -> &'static mut Arena {
        unsafe { &mut *(((self as *const _ as u32) & 0xfffff) as *mut Arena) }
    }
}

#[repr(C)]
pub struct Arena {
    /**
     * 该arena为哪个容器提供的内存块
     */
    supply_for: *mut MemBlockContainer,
    /**
     * 该arena内每个内存块的大小
     */
    pub block_size: usize,
    /**
     * 该arena未使用的内存块数量
     */
    pub left_block_cnt: usize,
}
impl Arena {
    pub fn init(&mut self, supply_for: *mut MemBlockContainer, block_size: usize, left_block_cnt: usize) {
        self.supply_for = supply_for;
        self.block_size = block_size;
        self.left_block_cnt = left_block_cnt;
    }
    /**
     * 根据内存块的下标，在当前Arena中找到这个内存块
     */
    pub fn find_mem_block(&self, block_idx: usize) -> &'static mut MemBlock {
        let mem_addr = (self as *const _ as usize) + size_of::<Arena>() + block_idx * size_of::<MemBlock>();
        unsafe { &mut *(mem_addr as *mut MemBlock) }
    }
    
    pub fn apply_one(&mut self) {
        self.left_block_cnt -= 1;
    }
    
}