use core::{mem::size_of, ptr};


use os_in_rust_common::{instruction, paging::{PageTable, PageTableEntry}, printk, printkln, ASSERT};

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
#[inline(never)]
pub fn add_page_connection(virtual_addr: usize, physical_addr: usize) {
    let pde =  addr_to_pde(virtual_addr);
    let pte =  addr_to_pte(virtual_addr);
    // 填充pde和pte和物理地址的关系。并且返回 是否新创建了页表
    self::do_add_page_connection(pde, pte, physical_addr);
}


/**
 * 已知pde和pte，我们给定物理地址，然后填充到页目录表和页表中
 */
pub fn do_add_page_connection(pde: &mut PageTableEntry, pte: &mut PageTableEntry, physical_addr: usize) {
    // 如果PDE已经赋值过了
    if pde.present() {
        // PTE一定要没有赋值过
        ASSERT!(!pte.present());
        // 构建一个PTE，塞进去
        *pte = PageTableEntry::new_default(physical_addr);
    }

    // 如果PDE没有赋值，从内核内存池中申请1页
    let kernel_page_table_addr =  memory::memory_poll::get_kernel_mem_pool().apply_one().unwrap();
    // 把页表的地址，赋值给这个页目录项
    *pde = PageTableEntry::new_default(kernel_page_table_addr);

    // 然后把我们的物理地址，赋值给这个新页表的这一项
    *pte = PageTableEntry::new_default(physical_addr);
}

/**
 * 已知当前的虚拟地址，把该虚拟地址在当前PTE中的连接取消
 * （页表项的P位设置为0）
 */
pub fn unset_pte(virtual_addr: usize) {
    let pte = addr_to_pte(virtual_addr);
    pte.set_present(false);
    // 刷新TLB缓存
    instruction::invalidate_page(virtual_addr);
}

/**
 * 已知虚拟地址，取消页目录项的映射
 */
pub fn unset_pde(virtual_addr: usize) {
    let pde = self::addr_to_pde(virtual_addr);
    pde.set_present(false);
    instruction::invalidate_page(virtual_addr);
}

/**
 * 构造一个虚拟地址，可以访问到virtual_addr地址经过的那个页目录项自身
 * 
 * 为什么不直接访问页表呢？因为开启分页后，给定的虚拟地址都需要经过页表映射来访问。
 * 但是我们页表的物理地址是没有在页表中配置映射的，所以直接使用物理地址是无法访问的
 * 
 * 因此我们构建一个地址，高10位是1、中间10位也是1，低12位是虚拟地址的高10位
 *   - 构造的地址的高10位是中间10位都是1，那么访问页目录表两次
 *   - 构造的地址的低12位是虚拟地址的高10位，那么就会访问到该虚拟地址原本要经过的页目录项
 */
pub fn addr_to_pde(virtual_addr: usize) -> &'static mut PageTableEntry {
    // 高10位，作为目录项的下标
    let pde_idx = locate_pde(virtual_addr);
    // 构造一个地址，当访问这个地址的时候，可以访问到这个页目录项本身
    unsafe { &mut *((0xfffff000 + pde_idx * size_of::<PageTableEntry>() ) as *mut PageTableEntry) }
}


/**
 * 构建一个虚拟地址，可以访问到该虚拟地址经过的页表项自身
 */
pub fn addr_to_pte(virtual_addr: usize) -> &'static mut PageTableEntry {
    // 取虚拟地址的中间10位，就是pte所在页表的下标
    let pte_idx = self::locate_pte(virtual_addr);
    // 构造一个地址。高10位是1，中间10位是该地址的高10位，然后地址的中间10位作为下标
    unsafe { &mut *((0xffc00000 + ((virtual_addr & 0xffc00000) >> 10) + pte_idx * size_of::<PageTableEntry>()) as *mut PageTableEntry) }
}

/**
 * 一个可以访问到当前页目录表自身的地址
 */
pub fn addr_to_dir_table() -> *mut PageTable {
    // 当一个地址，高10位是1，中间10位是1，那么会回环两次，就会访问到页目录表自身
    0xfffff000 as *mut PageTable
}

/**
 * 经过页表索引，得到virtual_addr虚拟地址指向的物理地址
 */
pub fn get_phy_from_virtual_addr(virtual_addr: usize) -> usize {
    // 得到这个虚拟地址，会映射到的页表项，得到该页表项的虚拟地址
    let pte = addr_to_pte(virtual_addr);
    // 取出该页表项，页表项的结构中，高20位，就是物理页框的物理地址的高20位
    let frame_phy_addr = pte.get_data() & 0xfffff000;

    // 物理页框地址高20位 + 该虚拟地址低12位 = 该虚拟地址将要访问的物理地址
    return frame_phy_addr as usize + (virtual_addr & 0x00000fff);
}

/**
 * 已知虚拟地址，得到该虚拟地址对应的目录项所在页目录表的下标
 * 虚拟地址的高10位是页目录表下标
 */
pub fn locate_pde(vaddr: usize) -> usize {
    (vaddr & 0xffc00000) >> 22
}

/**
 * 已知虚拟地址，得到该虚拟地址对应的页表项所在页表的下标
 * 虚拟地址的中间10位是页目录表下标
 */
pub fn locate_pte(vaddr: usize) -> usize {
    (vaddr & 0x003ff000) >> 12
}