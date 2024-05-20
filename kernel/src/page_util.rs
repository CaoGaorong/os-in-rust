use core::mem::size_of;


use os_in_rust_common::{paging::PageTableEntry, ASSERT};

use crate::memory;

/**
 * 页表工具
 * 这里的工具是在**「开启分页之后」**的操作
 */

/**
 * 建立页表连接
 * virtual_addr: 要建立连接的虚拟地址
 * physical_addr: 要建立连接的物理地址
 * 
 * 这是建立在一个前提下，就是页目录表已经存在了。如果整个页目录表都不存在，那直接就page fault了
 */
pub fn add_page_connection(virtual_addr: u32, physical_addr: u32) {
    let pde_addr =  addr_to_pde(virtual_addr);
    let pte_addr =  addr_to_pte(virtual_addr);
    let pde = unsafe { *pde_addr };
    let pte = unsafe { *pte_addr };
    // 如果PDE已经赋值过了
    if pde.present() {
        // PTE一定要没有赋值过
        ASSERT!(!pte.present());
        // 构建一个PTE，塞进去
        unsafe { *pte_addr = PageTableEntry::new_default(physical_addr) };
        return;
    }

    // 如果PDE没有赋值，从内核内存池中申请1页
    let kernel_page_table_addr = unsafe { memory::KERNEL_MEM_POOL.get_mut() }.apply_one().unwrap();
    // 把页表的地址，赋值给这个页目录项
    unsafe { *pde_addr = PageTableEntry::new_default(kernel_page_table_addr) };
    // 然后把我们的物理地址，赋值给这个新页表的这一项
    unsafe { *pte_addr = PageTableEntry::new_default(physical_addr) };
}

/**
 * 根据虚拟地址，构建出一个可以访问到该虚拟地址经过的页目录项自身的地址
 * 为什么不直接访问页表呢？因为开启分页后
 * 
 */
fn addr_to_pde(virtual_addr: u32) -> *mut PageTableEntry {
    // 高10位，作为目录项的下标
    let pde_idx = (virtual_addr >> 22) as usize;
    // 构造一个地址，当访问这个地址的时候，可以访问到这个页目录项本身
    (0xfffff000 + pde_idx * size_of::<PageTableEntry>() ) as *mut PageTableEntry
}


/**
 * 构建一个虚拟地址，可以访问到该虚拟地址经过的页表项自身
 */
fn addr_to_pte(virtual_addr: u32) -> *mut PageTableEntry {
    // 取虚拟地址的中间10位，就是pte所在页表的下标
    let pte_idx = (virtual_addr & 0x003ff000) >> 12;
    // 构造一个地址。高10位是1，中间10位是该地址的高10位，然后地址的中间10位作为下标
    (0xffc00000 + ((virtual_addr & 0xffc00000) >> 10) + pte_idx * size_of::<PageTableEntry>() as u32) as *mut PageTableEntry
}

/**
 * 经过页表索引，得到virtual_addr虚拟地址指向的物理地址
 */
pub fn get_phy_from_virtual_addr(virtual_addr: u32) -> u32 {
    // 得到这个虚拟地址，会映射到的页表项，得到该页表项的虚拟地址
    let pte = unsafe { &mut *addr_to_pte(virtual_addr) };
    // 取出该页表项，页表项的结构中，高20位，就是物理页框的物理地址的高20位
    let frame_phy_addr = pte.get_data() & 0xfffff000;

    // 物理页框地址高20位 + 该虚拟地址低12位 = 该虚拟地址将要访问的物理地址
    return (frame_phy_addr + (virtual_addr & 0x00000fff));
}

