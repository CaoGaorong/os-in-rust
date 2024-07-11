#[cfg(test)]
mod tests {
    use std::{array, fs::{read, File}, io::{BufReader, Read, Seek}, mem::size_of, os::unix::fs::FileExt, slice};

    use kernel::{filesystem::{constant, dir::DirEntry, inode::{Inode, OpenedInode}, superblock::SuperBlock}};
    use os_in_rust_common::constants;

    const DISK_FILE_PATH: &str = "/Users/jackson/MyProjects/rust/os-in-rust/build/hd80M.img";

    #[test]
    fn test_opened_inode() {
        let opened_inode = OpenedInode::new(Inode::empty());
        println!("cache size:{}", opened_inode.get_data_blocks())
    }

    #[test]
    fn test_inode_size() {
        println!("inode size:{} ", size_of::<Inode>())
    }

    /**
     * 读取硬盘中的超级块
     */
    fn read_super_block() -> Box<[u8; size_of::<SuperBlock>()]> {
        let mut file = File::open(DISK_FILE_PATH).expect("failed to open disk file");
        file.seek(std::io::SeekFrom::Start((59895 * constants::DISK_SECTOR_SIZE) as u64)).expect("failed to seek");
        let mut reader = BufReader::new(&mut file);
        let mut buf = [0x0u8; size_of::<SuperBlock>()];
        reader.read(&mut buf).expect("failed to read file");
        Box::new(buf)
    }

    /**
     * 测试读取超级块
     */
    #[test]
    fn test_read_super_block() {
        let super_block = read_super_block();
        let super_block = super_block.as_ref();
        let super_block = unsafe { &*(super_block.as_ptr() as *const SuperBlock) };
        println!("{:?}", super_block);
    }

    /**
     * 读取inode列表
     */
    fn read_inode_table() -> Box<[u8; size_of::<Inode>() * constant::MAX_FILE_PER_FS as usize]> {
        let super_block = read_super_block();
        let super_block = super_block.as_ref();
        let super_block = unsafe { &*(super_block.as_ptr() as *const SuperBlock) };

        // 读取inode表
        let mut file = File::open(DISK_FILE_PATH).expect("failed to open hd80M.img");
        file.seek(std::io::SeekFrom::Start((usize::from(super_block.inode_table_lba) * constants::DISK_SECTOR_SIZE) as u64)).expect("failed to seek file");
        let mut reader = BufReader::new(&mut file);
        let mut arr = [0x00u8; size_of::<Inode>() * constant::MAX_FILE_PER_FS as usize];
        // 读取文件
        reader.read_exact(&mut arr).expect("failed to read data");
        
        Box::from(arr)
    }
    /**
     * 测试读取inode表
     */
    #[test]
    fn test_read_inode_table() {
        let inode_table = read_inode_table();
        let inode_table = inode_table.as_ref();
        let inode_table = unsafe { std::slice::from_raw_parts(inode_table.as_ptr() as *const Inode, inode_table.len() / size_of::<Inode>()) };
        inode_table.iter().filter(|inode| inode.i_size > 0).for_each(|inode| {
            println!("inode no: {:?}", inode.i_no);
            println!("{:?}", inode);
        });
    }

    /**
     * 测试读取下根目录内的所有目录项
     */
    #[test]
    fn test_read_root_dir() {
        // 读取出inode表
        let inode_table = read_inode_table();
        let inode_table = inode_table.as_ref();
        let inode_table = unsafe { std::slice::from_raw_parts(inode_table.as_ptr() as *const Inode, inode_table.len() / size_of::<Inode>()) };
        // 取出根目录inode
        let root_dir = inode_table[0];
        let opened_inode = OpenedInode::new(root_dir);
        let direct_data_block = opened_inode.get_direct_data_blocks_ref();
        let mut file = File::open(DISK_FILE_PATH).expect("failed to open disk file");

        // 遍历根目录的数据区
        for data_lba in direct_data_block {
            if data_lba.is_empty() {
                continue;
            }
            // 读取每一个数据区中的数据
            file.seek(std::io::SeekFrom::Start((data_lba.get_lba() as usize * constants::DISK_SECTOR_SIZE) as u64)).expect("failed to seek");
            let mut reader = BufReader::new(&mut file);
            let mut buf = [0x0u8; constants::DISK_SECTOR_SIZE];
            reader.read(&mut buf).expect("failed to read file");
            // 转成目录项的结构
            let dir_list = unsafe { slice::from_raw_parts(buf.as_ptr() as *const DirEntry, buf.len() / size_of::<DirEntry>()) };

            // 打印一下，看下结果
            dir_list.iter().filter(|entry| !entry.is_empty()).for_each(|e| {
                println!("{:?}, entry name: {}", e, e.get_name());
                println!("------");
            });
        }
    }
}