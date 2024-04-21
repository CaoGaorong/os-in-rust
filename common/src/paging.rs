use core::borrow::BorrowMut;
use core::mem::size_of;


use lazy_static::lazy_static;
use volatile::Volatile;
use crate::racy_cell::RacyCell;
use crate::{println, utils};
/**
 * 页目录表的地址
 * 低端1MB上
 */

// const PAGE_DIR_ADDR: u32 = 0x100000;
const PAGE_ENTRY_COUNT: usize = 1024;


lazy_static! {

/**
 * 页目录表
*/
    #[no_mangle]
    #[link_section = ".page_dir"]
    pub static ref PAGE_DIRECTORY: RacyCell<PageTable> = RacyCell::new(PageTable::empty());

    #[no_mangle]
    #[link_section = ".page_table_list"]
    pub static ref PAGE_TABLE_LIST: [PageTable; PAGE_ENTRY_COUNT] = [PageTable::empty(); PAGE_ENTRY_COUNT];

}




/**
 * 填充页目录表项
 * 只填充0和768项。指向0号页表
 */
#[no_mangle]
pub fn fill_table_directory() {
    let page_table0 = PageTableEntry::new_default(&PAGE_TABLE_LIST[0] as *const _ as u32);
    
    let dir_table_ref = unsafe { PAGE_DIRECTORY.get_mut() };

    // 页目录表第0项和第768项指向0号页表
    dir_table_ref.set_entry(0, page_table0);
    dir_table_ref.set_entry(768, page_table0);
    
    // 页目录表的最后一项，指向自己
    let self_entry = PageTableEntry::new_default(unsafe { PAGE_DIRECTORY.get_mut() } as *const _ as u32);
    dir_table_ref.set_entry(dir_table_ref.size() - 1, self_entry);
}
/**
 * 填充内核页目录项。
 * 填充从769到1022项
 */
pub fn fill_kernel_directory() {
    // 2号页表 
    let directory_table_ref = unsafe {
        PAGE_DIRECTORY.get_mut()
    };
    let mut page_table_idx = 2;
    for idx in 769 .. 1023 {
        // 找到2号页表
        let page_table = PAGE_TABLE_LIST[page_table_idx];
        // 把2号页表的地址，赋值给第idx页目录项
        directory_table_ref.set_entry(idx, PageTableEntry::new_default(&page_table as *const PageTable as u32));
        page_table_idx += 1;
    }
}

/**
 * 填充0号页表
 */
pub fn fill_table0() {
    // 一个页表项，指向4KB的物理空间
    const PAGE_SIZE: u32 = 4 * 1024;
    // 0号页表
    let page_table0 = PAGE_TABLE_LIST[0];
    let cell  = RacyCell::new(page_table0);
    for i in 0 .. 1024 {
        // 页表，指向从0开始的低端地址。
        let entry = PageTableEntry::new_default(i * PAGE_SIZE);
        unsafe { cell.get_mut().set_entry(i as usize, entry) }
    }
}




/**
 * 分页相关
 * 构建页表，然后把页表的地址加载到cr3寄存器
 */

const PAGE_TABLE_ENTRY_COUNT: usize = 1024;
/**
 * 页表
 * 一个页表包含1024个页表项
 */
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTable {
    data: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT]
}
impl PageTable {
    /**
     * 得到一个空的页表（创建1024项的空间）
     */
    pub const fn empty() -> Self {
        Self {
            data: [PageTableEntry::empty(); PAGE_TABLE_ENTRY_COUNT]
        }
    }
    pub fn from(addr: u32) -> &'static mut Self {
        unsafe {
            &mut *(addr as *mut Self)
        }
    }
    /**
     * 给该页表的的下标为index的页表项赋值
     */
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.data[index] = entry;
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/**
 * 页表项的结构：<https://wiki.osdev.org/Page_table#Page_Table>
 */
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    /**
     * 内容是4个字节，数据比较散，就不分成多个字段了
     */
    data: u32
}

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self { data: 0 }
    }
    /**
     * 32位的内存地址
     */
    pub fn new_default(address: u32) -> Self {
        Self::new(
            address, 
            true, 
            true, 
            true, 
            false,
            false, 
            false, 
            false, 
            false, 
            false)
    }
    /**
     * address: 该页表项指向的地址的高20位
     */
    pub fn new(
        address: u32,
        present: bool, 
        wr_enable: bool, 
        user: bool, 
        page_write_through:bool, 
        page_cache_enable: bool, 
        access: bool,
        dirty: bool,
        page_attribute_table:bool,
        global: bool,
    ) -> Self {

        Self { 
            data: 
                ((address >> 12) as u32) << 12 | 
                (utils::bool_to_int(present)) |
                (utils::bool_to_int(wr_enable) << 1) | 
                (utils::bool_to_int(user) << 2) |
                (utils::bool_to_int(page_write_through) << 3) |
                (utils::bool_to_int(page_cache_enable) << 4) |
                (utils::bool_to_int(access) << 5) |
                (utils::bool_to_int(dirty) << 6) |
                (utils::bool_to_int(page_attribute_table) << 7) |
                (utils::bool_to_int(global) << 8) |
                (utils::bool_to_int(global) << 9) | 
                (0 << 11)
        }
    }
}

