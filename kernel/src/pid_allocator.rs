use os_in_rust_common::{bitmap::{self, BitMap}, pool::MemPool, ASSERT};

use crate::mutex::Mutex;


/**
 * pid池子
 */
struct PidPool {
    /**
     * pid池子位图
     */
    bitmap: bitmap::BitMap,
    /**
     * 起始的pid号
     */
    start_pid: u8,
}

impl PidPool {
    pub const fn empty() -> Self {
        Self {
            bitmap:BitMap::new(&PID_BITS),
            start_pid: 10
        }
    }
}
const PID_BITS: [u8; 128] = [0; 128];

static mut GLOBAL_PID_POOL: Mutex<PidPool> = Mutex::new(PidPool::empty());


/**
 * 从位图里面申请一个pid
 */
pub fn allocate() -> u8 {
    let mut pid_pool = unsafe { GLOBAL_PID_POOL.lock() };
    // 从位图里面找一位
    let res = pid_pool.bitmap.apply_bits(1);
    ASSERT!(res.is_ok());
    let bit_idx = res.unwrap();
    
    // 设置这一位，已占用
    pid_pool.bitmap.set_bit(bit_idx, true);

    // 开始的pid + 位图申请到的位下标
    pid_pool.start_pid + bit_idx as u8
}


/**
 * 释放某一个pid
 */
pub fn release(pid_to_release: u8) {
    let mut pid_pool = unsafe { GLOBAL_PID_POOL.lock() };
    let bit_idx = pid_to_release - pid_pool.start_pid;
    // 把那一位设置为 未使用
    pid_pool.bitmap.set_bit(bit_idx as usize, false);
}
