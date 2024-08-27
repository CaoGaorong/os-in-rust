#[cfg(test)]
mod tests {
    use std::{fs::File, mem::size_of, slice};

    use kernel::filesystem::{DirEntry, inode::{Inode, OpenedInode}, superblock::SuperBlock};
    use tests::file_system;
    

    const DISK_FILE_PATH: &str = "/Users/jackson/MyProjects/rust/os-in-rust/build/hd80M.img";

    #[test]
    fn test_opened_inode() {
        let opened_inode = OpenedInode::new(Inode::empty());
        println!("cache size:{}", opened_inode.get_data_blocks_ref().len())
    }

    #[test]
    fn test_inode_size() {
        println!("inode size:{} ", size_of::<Inode>());
        println!("OpenedInode size:{} ", size_of::<OpenedInode>());
        println!("DirEntry size:{} ", size_of::<DirEntry>());
    }


    /**
     * 测试读取超级块
     */
    #[test]
    fn test_read_super_block() {
        let super_block = file_system::read_super_block();
        let super_block = super_block.as_ref();
        let super_block = unsafe { &*(super_block.as_ptr() as *const SuperBlock) };
        println!("{:?}", super_block);
    }

    /**
     * 测试读取inode表
     */
    #[test]
    fn test_read_inode_table() {
        let inode_table = file_system::read_inode_table();
        let inode_table = inode_table.as_ref();
        let inode_table = unsafe { std::slice::from_raw_parts(inode_table.as_ptr() as *const Inode, inode_table.len() / size_of::<Inode>()) };
        inode_table.iter().filter(|&&inode| inode.i_size > 0).for_each(|inode| {
            println!("inode no: {:?}", inode.i_no);
            println!("{:?}", inode);
        });
    }


    #[test]
    fn test_load_inode() {
        let mut disk_file = File::open(DISK_FILE_PATH).expect("failed to open disk file");
        let inode_table = file_system::read_inode_table();
        let inode_table = inode_table.as_ref();
        let inode_table = unsafe { std::slice::from_raw_parts(inode_table.as_ptr() as *const Inode, inode_table.len() / size_of::<Inode>()) };
        let root_inode = inode_table[0];
        let opened_root_inode = file_system::load_inode(&mut disk_file, root_inode);
        println!("{:?}", opened_root_inode)
    }
}