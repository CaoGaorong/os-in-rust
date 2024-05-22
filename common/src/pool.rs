use core::mem::size_of;

use crate::{bitmap::{BitMap, MemoryError}, println};

pub struct MemPool {
    /**
     * 内存池的位图
     */
    pub bitmap: BitMap,
    /**
     * 该内存池，对应的地址起始地址
     */
    pub addr_start: usize,
    
    /**
     * 该内存池位图，1位的粒度
     */
    pub granularity: usize,
}

unsafe impl Send for MemPool{}
unsafe impl Sync for MemPool {}

impl MemPool {
    pub fn new(addr_start: usize, bitmap: BitMap) -> Self {
        Self {
            addr_start, bitmap, granularity: 4 * 1024
        }
    }

    pub const fn empty() -> Self {
        Self {
            addr_start: 0,
            bitmap: BitMap::empty(),
            granularity: 0,
        }
    }

    /**
     * 填充
     * addr_start: 该内存池描述的内存块的起始地址
     * granularity: 该内存池位图的粒度
     * bitmap: 该内存池位图所在的内存数组
     */
    pub fn init(&mut self, addr_start: usize, granularity: usize, bitmap: &'static mut [u8]) {
        self.addr_start = addr_start;
        self.granularity = granularity;
        self.bitmap.map_ptr = bitmap.as_mut_ptr();
        self.bitmap.size = bitmap.len();
        // self.bitmap.clear();
    }
    /**
     * 申请page_cnt页的空间大小，返回申请到的该页的起始虚拟地址
     */
    pub fn apply(&mut self, page_cnt: usize) -> Result<usize, MemoryError> {
        
        // 从位图里面，申请连续page_cnt位
        let bit_idx_available = self.bitmap.apply_bits(page_cnt)?;
        // 然后把位图的申请到的位，设置为true
        for i in 0 .. page_cnt {
            self.bitmap.set_bit(bit_idx_available + i, true);
        }
        
        // 返回虚拟地址。位图每一位，粒度是granularity
        Result::Ok(self.addr_start + bit_idx_available * self.granularity)
    }

    /**
     * 申请1个。得到申请到的起始虚拟地址
     */
    pub fn apply_one(&mut self) -> Result<usize, MemoryError> {
        self.apply(1)
    }
}