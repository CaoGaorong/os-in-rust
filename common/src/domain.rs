use core::{fmt::Display, ops::{Add, Sub}};

use crate::printkln;


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
    #[inline(never)]
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
     * 当前LBA地址是否为空
     */
    pub fn is_empty(&self) -> bool {
        if self.data <= 0 {
            return true;
        }
        return false;
    }
    
}

impl Add for LbaAddr {
    type Output = LbaAddr;

    fn add(self, rhs: Self) -> Self::Output {
        LbaAddr::new(self.data  + rhs.data)
    }
}

impl Sub for LbaAddr {
    type Output = LbaAddr;

    fn sub(self, rhs: Self) -> Self::Output {
        LbaAddr::new(self.data - rhs.data)
    }
}

impl From<u32> for LbaAddr {
    fn from(value: u32) -> Self {
        Self {
            data: value,
        }
    }
}
impl From<LbaAddr> for usize {
    fn from(value: LbaAddr) -> Self {
        value.data.try_into().unwrap()
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct InodeNo {
    data: u32,
}
impl InodeNo {
    pub fn new(idx: u32) -> Self {
        Self {
            data: idx,
        }
    }

    pub fn add(&self, offset: usize) -> Self {
        Self::new(self.data + offset as u32)
    }
}

impl Display for InodeNo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        printkln!("{}", self.data as u32);
        return Result::Ok(());
    }
}

impl Sub for InodeNo {
    type Output = InodeNo;

    fn sub(self, rhs: Self) -> Self::Output {
        InodeNo::new(self.data - rhs.data)
    }
}

impl PartialEq for InodeNo {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl From<usize> for InodeNo {
    fn from(value: usize) -> Self {
        InodeNo::new(value as u32)
    }
}

impl From<u32> for InodeNo {
    fn from(value: u32) -> Self {
        InodeNo::new(value)
    }
}

impl From<InodeNo> for u32 {
    fn from(value: InodeNo) -> Self {
        value.data
    }
}
impl From<InodeNo> for usize {
    fn from(value: InodeNo) -> Self {
        value.data.try_into().unwrap()
    }
}


