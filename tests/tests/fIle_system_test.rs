#![feature(abi_x86_interrupt)]
#[cfg(test)]
mod tests {
    use kernel::filesystem::superblock::SuperBlock;
    use os_in_rust_common::domain::LbaAddr;

    #[test]
    fn test_super_block() {
        let super_block = SuperBlock::new(LbaAddr::new(0x231), 0x3123123);
        println!("{:?}", super_block);
        println!("hello");
    }
}