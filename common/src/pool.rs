use crate::bitmap::{BitMap, MemoryError};
pub struct MemPool {
    /**
     * 内存池的位图
     */
    bitmap: BitMap,
    /**
     * 该内存池，对应的地址起始地址
     */
    addr_start: usize,
    
    /**
     * 该内存池位图，1位的粒度
     */
    granularity: usize,
}

impl MemPool {
    pub fn new(addr_start: usize, bitmap: BitMap) -> Self {
        Self {
            addr_start, bitmap, granularity: 4 * 1024
        }
    }
    /**
     * 申请page_cnt页的空间大小，返回申请到的该页的起始虚拟地址
     */
    pub fn apply(&mut self, page_cnt: usize) -> Result<usize, MemoryError> {
        
        // 从位图里面，申请连续page_cnt位
        let bit_idx_avaliable = self.bitmap.apply_bits(page_cnt)?;
        
        // 然后把位图的申请到的位，设置为true
        for i in 0 .. page_cnt {
            self.bitmap.set_bit(bit_idx_avaliable + i, true);
        }
        
        // 返回虚拟地址。位图每一位，粒度是granularity
        Result::Ok(self.addr_start + bit_idx_avaliable * self.granularity)
    }

    /**
     * 申请1个。得到申请到的起始虚拟地址
     */
    pub fn apply_one(&mut self) -> Result<usize, MemoryError> {
        self.apply(1)
    }
}