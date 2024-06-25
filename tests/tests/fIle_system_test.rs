#![feature(abi_x86_interrupt)]
#[cfg(test)]
mod tests {
    use kernel::filesystem::superblock::SuperBlock;

    #[test]
    fn test_super_block() {
        let super_block = SuperBlock::new(0x231, 0x3123123);
        println!("{:?}", super_block);
        println!("hello");
    }
}