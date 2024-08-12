
use core::{fmt::Display, ptr};

use crate::{printkln, ASSERT};

#[derive(Debug)]
#[repr(C, packed)]
pub struct BitMap {
    /**
     * 位图的数据指针
     */
    pub map_ptr: *mut u8,
    /**
     * 位图的大小；单位：字节
     */
    pub size: usize,
    init: bool,
}

// 自己保证并发问题
unsafe impl Send for BitMap {}
unsafe impl Sync for BitMap {}


#[derive(Debug)]
pub enum MemoryError {
    MemInsufficient,
    NoMemeoryMap,

}

impl BitMap {
    /**
     * 构建一个位图。把一个数组，转换成一个位图
     */
    pub const fn new(bitmap: &mut [u8]) -> Self {
        Self {
            map_ptr: bitmap.as_mut_ptr(),
            size: bitmap.len(),
            init: true
        }
    }
    // 一个空值
    pub const fn empty() -> Self {
        Self {
            map_ptr: ptr::null_mut(),
            size: 0,
            init: false
        }
    }

    pub fn init(&mut self, bitmap: &[u8]) {
        self.map_ptr = bitmap as *const [u8] as *mut u8;
        self.size = bitmap.len();
        self.init = true; // 设置为初始化过了
    }
    /**
     * 把位图指向的空间清零
     */
    pub fn clear(&self) {
        ASSERT!(self.init);
        unsafe { ptr::write_bytes(self.map_ptr, 0, self.size) }
    }
    /**
     * 在位图中申请cnt个连续的bit。返回申请到的bit在该位图的下标
     */
    #[inline(never)]
    pub fn apply_bits(&self, cnt: usize) -> Result<usize, MemoryError>{
        ASSERT!(self.init);
        let mut successive_cnt = 0;
        // 遍历位图的每一个字节
        for i in  0..self.size {
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
                    return Result::Ok((i * 8 + j + 1) - successive_cnt);
                }
            }

        }
        Result::Err(MemoryError::MemInsufficient)
    }
    /**
     * 设置某一位
     */
    pub fn set_bit(&mut self, bit_idx: usize, val: bool) {
        ASSERT!(self.init);
        ASSERT!(bit_idx <= self.size * 8);

        let byte_idx: isize = bit_idx as isize / 8;
        let bit_offset = bit_idx % 8;

        if val {
            unsafe { *self.map_ptr.offset(byte_idx) |=  1 << bit_offset };
        } else {
            unsafe { *self.map_ptr.offset(byte_idx) &= !(1 << bit_offset) };
        }
    }

    /**
     * 某一bit的下标，该位是否被set为1
     */
    #[inline(never)]
    pub fn is_set(&self, bit_idx: usize) -> bool {
        ASSERT!(self.init);
        ASSERT!(bit_idx <= self.size * 8);

        // 转成字节数组
        let bitmap = unsafe { core::slice::from_raw_parts(self.map_ptr, self.size) };
        // 该位所在的字节
        let byte_idx = bit_idx / 8;
        // 定位到那一个字节
        let byte = bitmap[byte_idx];
        if byte == 0 {
            return false;
        }
        
        // 该位所在字节内的偏移
        let bit_offset = bit_idx % 8;

        return byte & (1 << bit_offset) == (1 << bit_offset);
    }

    /**
     * 得到bitmap信息
     */
    pub fn get_bitmap(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.map_ptr, self.size) }
    }

    /**
     * 该位图中，一共位的长度
     */
    pub fn bits_len(&self) -> usize {
        self.size * 8
    }


}

impl Display for BitMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        printkln!("BitMap(map_ptr:0x{:x}, size:{})", self.map_ptr as usize, self.size as u32);
        Result::Ok(())
    }
}