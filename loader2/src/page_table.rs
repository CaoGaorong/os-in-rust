
use os_in_rust_common::{constants, paging::{PageTable, PageTableEntry}, racy_cell::RacyCell};


/**
 * 页目录表
*/
#[no_mangle]
#[link_section = ".page_dir"]
pub static  PAGE_DIRECTORY: RacyCell<PageTable> = RacyCell::new(PageTable::empty());

/**
 * 内核页表：255项
 *  * dir[0] = table[0]
 *  * dir[768] = table[0]
 *  * dir[769] = table[1]
 *  * ......
 *  * dir[1022] = table[254]
 */
#[no_mangle]
#[link_section = ".page_table_list"]
pub static mut PAGE_TABLE_LIST: [PageTable; constants::KERNEL_PAGE_TABLE_CNT] = [PageTable::empty(); constants::KERNEL_PAGE_TABLE_CNT];

/**
 * 获取页目录表的地址。只有在未开启分页，才能使用
 */
pub fn get_dir_ref() -> &'static PageTable {
    unsafe { PAGE_DIRECTORY.get_mut() }
}

/**
 * 获取页表的地址。只有未开启分页，才能使用
 */
pub fn get_table_list_ref() -> &'static [PageTable] {
    unsafe { &PAGE_TABLE_LIST[..PAGE_TABLE_LIST.len()] }   
}

fn get_table_ref(idx: usize) -> &'static mut PageTable {
    unsafe { &mut PAGE_TABLE_LIST[idx] }
}




/**
 * 填充页目录表项
 * 只填充0和768项。指向0号页表
 */
#[no_mangle]
pub fn  fill_dir_directory() {
    let page_table0 = PageTableEntry::new_default(get_table_ref(0) as *const _ as usize);
    
    let dir_table_ref = unsafe { PAGE_DIRECTORY.get_mut() };

    // 页目录表第0项和第768项指向0号页表
    dir_table_ref.set_entry(0, page_table0);
    dir_table_ref.set_entry(constants::KERNEL_DIR_ENTRY_START_IDX, page_table0);
    
    // 页目录表的最后一项，指向自己
    let self_entry = PageTableEntry::new_default(dir_table_ref as *const PageTable as usize);
    dir_table_ref.set_entry(dir_table_ref.size() - 1, self_entry);
}
/**
 * 填充内核页目录项。
 * 填充从769到1022项
 */
#[no_mangle]
pub fn fill_kernel_directory() {
    // 2号页表 
    let directory_table_ref = unsafe {
        PAGE_DIRECTORY.get_mut()
    };
    let mut page_table_idx = 1;

    // 遍历页目录项。页目录项下标范围是：[769, 1023)
    for dir_entry_idx in constants::KERNEL_DIR_ENTRY_START_IDX + 1 .. constants::KERNEL_DIR_ENTRY_END_IDX {
        // 找到page_table_idx号页表
        let page_idx_addr = get_table_ref(page_table_idx);
        // 把page_table_idx号页表的地址，赋值给第idx页目录项
        directory_table_ref.set_entry(dir_entry_idx, PageTableEntry::new_default(page_idx_addr as *const PageTable as usize));
        page_table_idx += 1;
    }
}

/**
 * 填充0号页表
 */
#[inline]
#[no_mangle]
pub fn fill_table0(){
    // 一个页表项，指向4KB的物理空间
    const PAGE_SIZE: u32 = constants::PAGE_SIZE;
    let page_table_0 = get_table_ref(0);

    // 0号页表
    for i in 0 .. 256 {
        // 页表，指向从0开始的低端地址。
        let entry = PageTableEntry::new_default(i * PAGE_SIZE as usize);
        page_table_0.set_entry(i as usize, entry);
    }
}



