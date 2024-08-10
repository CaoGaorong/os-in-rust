pub mod test {
    use os_in_rust_common::{bitmap::BitMap, pool::MemPool};

    #[test]
    pub fn test_mempool_iterator() {
        let bits = [1u8, 2 , 3 , 4];
        let ptr = unsafe { *(&bits as *const _ as *const u32) };
        println!("bits: {:b}", ptr);
        let pool = MemPool::new(0xabcd, BitMap::new(&bits));
        for vaddr in pool.iter() {
            println!("addr: 0x{:x}", vaddr);
        }
    }
}