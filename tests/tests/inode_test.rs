#[cfg(test)]
mod tests {
    use kernel::filesystem::inode::{Inode, OpenedInode};

    #[test]
    fn test_opened_inode() {
        let opened_inode = OpenedInode::new(Inode::empty());
        println!("cache size:{}", opened_inode.data_block_lba.len())
    }
}