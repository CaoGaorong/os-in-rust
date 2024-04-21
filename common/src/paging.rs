use core::mem::size_of;

use crate::racy_cell::RacyCell;
use crate::utils;
/**
 * 页目录表的地址
 * 低端1MB上
 */

// const PAGE_DIR_ADDR: u32 = 0x100000;
const PAGE_ENTRY_COUNT: usize = 1024;


/**
 * 页目录表
 */
#[no_mangle]
#[used]
#[link_section = ".page_dir"]
static PAGE_DIRECTORY_SPACE:[u32; PAGE_ENTRY_COUNT] = [0; PAGE_ENTRY_COUNT];


#[no_mangle]
#[link_section = ".page_table_list"]
static PAGE_TABLE_0_SPACE: [u32; PAGE_ENTRY_COUNT * PAGE_ENTRY_COUNT] = [0; PAGE_ENTRY_COUNT * PAGE_ENTRY_COUNT];





/**
 * 填充页目录表项
 * 只填充0和768项。指向0号页表
 */
#[no_mangle]
pub fn fill_table_directory() {
    static DIRECTORY_TABLE:RacyCell<PageTable> = RacyCell::new(unsafe { core::ptr::read(&PAGE_DIRECTORY_SPACE as *const _ as *const PageTable) });
    let page_table0 = PageTableEntry::new_default(&PAGE_TABLE_0_SPACE as *const _ as u32);
    // let directory_table_ref = PageTable::from(&PAGE_DIRECTORY_SPACE  as *const _ as u32);
    
    let dir_table_ref = unsafe { DIRECTORY_TABLE.get_mut() };

    // 页目录表的地址
    let dir_table_addr = dir_table_ref as *const PageTable as *const u8 as u32;

    // 第0个页目录项，填充
    unsafe { *(dir_table_addr as *mut PageTableEntry) = page_table0 };
    
    // 第0xc00项，填充
    let addr_0xc00: u32 = dir_table_addr + 0xc00 * size_of::<PageTableEntry> as u32;
    unsafe { *(addr_0xc00 as *mut PageTableEntry) = page_table0 };

    // 页目录表第0项和第0xc00项指向0号页表
    // dir_table_ref.set_entry(0x0, page_table0);
    // dir_table_ref.set_entry(0xc00, page_table0);

    // 页目录表的最后一项，指向自己
    // let self_entry = PageTableEntry::new_default(&PAGE_DIRECTORY_SPACE as *const _ as u32);
    // dir_table_ref.set_entry(dir_table_ref.size() - 1, self_entry);
    // println!("Fuck");
}
/**
 * 填充内核页目录项。
 * 填充从769到1022项
 */
// pub fn fill_kernel_directory() {
//     // 2号页表 
//     let mut page_table_idx = 2;
//     let directory_table_ref = unsafe {
//         PAGE_DIRECTORY_TABLE.get_mut()
//     };
//     for idx in 769 .. 1023 {
//         // 找到2号页表
//         let page_table = PAGE_TABLE_LIST[page_table_idx];
//         // 把2号页表的地址，赋值给第idx页目录项
//         directory_table_ref.set_entry(idx, PageTableEntry::new_default(&page_table as *const PageTable as u32));
//         page_table_idx += 1;
//     }
// }

/**
 * 填充0号页表
 * 
 */
// pub fn fill_table0() {
//     // 一个页表项，指向4KB的物理空间
//     const PAGE_SIZE: u32 = 4 * 1024;
//     // 0号页表
//     let page_table0 = PAGE_TABLE_LIST[0];
//     for i in 0 .. 1024 {
//         // 页表，指向从0开始的低端地址。
//         let entry = PageTableEntry::new_default(i * PAGE_SIZE);
//         page_table0.set_entry(i as usize, entry)
//     }
// }




/**
 * 分页相关
 * 构建页表，然后把页表的地址加载到cr3寄存器
 */

const PAGE_TABLE_ENTRY_COUNT: usize = 1024;
/**
 * 页表
 * 一个页表包含1024个页表项
 */
#[repr(C)]
pub struct PageTable {
    data: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT]
}
impl PageTable {
    /**
     * 得到一个空的页表（创建1024项的空间）
     */
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    /**
     * 内容是4个字节，数据比较散，就不分成多个字段了
     */
    data: u32
}

impl PageTableEntry {
    pub fn empty() -> Self {
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
