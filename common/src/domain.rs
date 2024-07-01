use core::{fmt::Display, ops::Add};


/**
 * LBA地址
 * 由于用纯数字，在传递过程中，不知道语义了
 */
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct LbaAddr {
    data: u32
}
impl LbaAddr {
    pub const fn new(addr: u32) -> Self {
        Self {
            data: addr
        }
    }

    pub const fn empty() -> Self {
        Self::new(0)
    }

    /**
     * 累加一个地址。不改变原地址的值
     */
    pub fn add(&self, addr: u32) -> Self {
        Self {
            data: self.data + addr,
        }
    }
    /**
     * 获取LBA地址
     */
    pub fn get_lba(&self) -> u32 {
        self.data
    }
    /**
     * 获取一个合法的值
     */
    pub fn is_valid(&self) -> bool {
        if self.data <= 0 {
            return false;
        }
        return true;
    }
    
}

impl Add for LbaAddr {
    type Output = LbaAddr;

    fn add(self, rhs: Self) -> Self::Output {
        LbaAddr::new(self.data  + rhs.data)
    }
}