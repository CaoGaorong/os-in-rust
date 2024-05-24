
use crate::racy_cell::RacyCell;
use crate::{constants, utils};

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
    /**
     * 根据一个地址，把这地址的空间，强行转成页表的格式
     */
    pub fn from(addr: usize) -> &'static mut Self {
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

    /**
     * 得到某一目录项
     */
    pub fn get_entry(&self, index: usize) -> &PageTableEntry {
        &self.data[index]
    }

    /**
     * 从某一个页表中赋值若干项页目录项到当前页表中
     * from_table: &PageTable 从这个页表复制数据
     * from_idx: usize 要复制页目录项所在页表的起始下标
     * len: usize 要复制页目录项的数量
     */
    pub fn copy_from(&mut self, from_table: &PageTable, from_idx: usize, len: usize) {
        for idx in from_idx .. from_idx + len {
            // 从from_table取出这一项，然后复制给当前的页表
            self.data[idx] = *(from_table.get_entry(idx));
        }
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
    pub fn new_default(address: usize) -> Self {
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
        address: usize,
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
                (((address >> 12) as u32) << 12) | 
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
    pub fn present(&self) -> bool {
        self.data & 0x00000001 == 0x1
    }
    pub fn get_data(&self) -> u32 {
        self.data
    }
}

