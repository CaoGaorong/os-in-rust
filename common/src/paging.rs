use core::mem::size_of;

use crate::{constants, utils};

pub struct PageTableEntryIterator<'a> {
    pgdir: &'a PageTable,
    idx: usize,
}
impl <'a> Iterator for PageTableEntryIterator<'a> {
    type Item = &'a PageTableEntry;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.pgdir.size() {
            return Option::None;
        }
        let target = self.pgdir.get_entry(self.idx);
        self.idx += 1;
        return Option::Some(&target);
    }
}

 /**
  * 页表
  * 一个页表包含1024个页表项
  */
 #[repr(transparent)]
 #[derive(Clone, Copy)]
 pub struct PageTable {
     data: [PageTableEntry; constants::PAGE_TABLE_ENTRY_COUNT]
 }
 impl PageTable {
     /**
      * 得到一个空的页表（创建1024项的空间）
      */
     pub const fn empty() -> Self {
         Self {
             data: [PageTableEntry::empty(); constants::PAGE_TABLE_ENTRY_COUNT]
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
    #[inline(never)]
     pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
         self.data[index] = entry;
     }
 
     /**
      * 得到某一目录项
      */
    #[inline(never)]
     pub fn get_entry(&self, index: usize) -> &PageTableEntry {
        &self.data[index]
     }

     #[inline(never)]
     pub fn get_entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.data[index]
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

     pub fn iter(&self) -> PageTableEntryIterator {
        PageTableEntryIterator { 
            pgdir: self, 
            idx: 0 
        }
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
     
     #[inline(never)]
     pub fn present(&self) -> bool {
         self.data & 0x00000001 == 0x1
     }

     pub fn set_present(&mut self, present: bool) {
        if present {
            self.data |= 0x1;
        } else {
            self.data &= !0x1;
        }
     }
    pub fn get_data(&self) -> u32 {
        self.data
    }
    
    /**
     * 获取物理地址（页表项的高20位就是物理地址）
     */
    pub fn get_phy_addr(&self) -> u32 {
        self.data & 0xfffff000
    }

    pub fn parse_table(&mut self, idx: usize) -> &mut PageTable {
        // 当前项的地址 - 前面的项的个数 * 项大小 = 页表的起始地址
        unsafe { &mut *(((self as *mut _ as usize) - (idx * size_of::<PageTableEntry>())) as *mut PageTable) }
    }

 }
 