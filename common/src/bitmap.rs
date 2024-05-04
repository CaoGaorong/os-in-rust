
use crate::ASSERT;

pub struct BitMap {
    /**
     * 位图的数据指针
     */
    map_ptr: *mut u8,
    /**
     * 位图的长度；单位：位
     */
    len: usize
}
#[derive(Debug)]
pub enum MemoryError {
    MemInsuffient,
    NoMemeoryMap,

}

impl BitMap {
    /**
     * 构建一个位图。把一个数组，转换成一个位图
     */
    pub const fn new(bitmap: &[u8]) -> Self {
        Self {
            map_ptr: bitmap as *const [u8] as *mut u8,
            len: bitmap.len() * 8
        }
    }
    /**
     * 在位图中申请cnt个连续的bit。返回申请到的bit在该位图的下标
     */
    pub fn apply_bits(&self, cnt: usize) -> Result<usize, MemoryError>{
        // 位图的长度，换算成字节
        let len_in_byte = self.len / 8;
        
        let mut successive_cnt = 0;
        // 遍历位图的每一个字节
        for i in  0..len_in_byte {
            // 取出每一个字节
            let byte = unsafe { *self.map_ptr.offset(i as isize) };
            // 如果这个字节每一位都满了，那么下一个字节
            if byte == 0xff {
                continue;
            }
            // 遍历每一位
            for j in 0..8 {
                if (byte >> j) & 0x1 == 0x0 {
                    successive_cnt += 1;
                } else {
                    successive_cnt = 0;
                }
                if successive_cnt == cnt {
                    return Result::Ok((i * 8 + j) - successive_cnt);
                }
            }

        }
        Result::Err(MemoryError::MemInsuffient)
    }
    /**
     * 设置某一位
     */
    pub fn set_bit(&mut self, bit_idx: usize, val: bool) {

        ASSERT!(bit_idx <= self.len);

        let byte_idx: isize = bit_idx as isize / 8;
        let bit_offset = bit_idx % 8;

        if val {
            unsafe { *self.map_ptr.offset(byte_idx) |=  1 << bit_offset };
        } else {
            unsafe { *self.map_ptr.offset(byte_idx) &= !(1 << bit_offset) };
        }
    }


}