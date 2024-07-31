mod test {
    use core::slice;
    use std::{fs::File, io::{BufReader, Read, Seek}, mem::size_of};

    use kernel::filesystem::{inode::{Inode, OpenedInode}, DirEntry, FileType};
    use os_in_rust_common::{constants, utils};
    use tests::file_system::{self, DISK_FILE_PATH};

    #[test]
    fn test_split() {
        let mut entry_list = "/dev/proc/id".split("/");
        while let Some(entry) = entry_list.next() {
            if entry.is_empty() {
                continue;
            }
            println!("entry: {:?}", entry);
        }
    }

    #[test]
    fn entry_count_in_sector() {
        println!("{}", constants::DISK_SECTOR_SIZE / size_of::<DirEntry>());
        println!("{}", utils::div_ceil(constants::DISK_SECTOR_SIZE as u32, size_of::<DirEntry>() as u32) as usize );
    }


    /**
     * 测试读取下根目录内的所有目录项
     */
    #[test]
    fn test_read_root_dir() {
        // 读取出inode表
        let inode_table =  file_system::read_inode_table();
        let inode_table = inode_table.as_ref();
        let inode_table = unsafe { std::slice::from_raw_parts(inode_table.as_ptr() as *const Inode, inode_table.len() / size_of::<Inode>()) };
        // 取出根目录inode
        let root_dir = inode_table[0];
        let opened_inode = OpenedInode::new(root_dir);
        let direct_data_block = opened_inode.get_direct_data_blocks_ref();
        let mut file = File::open(DISK_FILE_PATH).expect("failed to open disk file");

        // 遍历根目录的数据区
        for (idx, data_lba) in direct_data_block.iter().enumerate() {
            if data_lba.is_empty() {
                println!("idx: {}, lba:{:?} is empty", idx, *data_lba);
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
            dir_list.iter().filter(|entry| !entry.is_empty() ).for_each(|e| {
                println!("entry name: {}\n entry: {:?}\n  inode: {:?}\n", e.get_name(), e, inode_table[usize::from(e.i_no)]);
                println!("------");
            });
        }
    }

    #[test]
    pub fn read_dir_entry() {
        let entry_list = file_system::read_dir_entry("/");
        if entry_list.is_none() {
            println!("目录不存在");
            return;
        }
        let entry_list = entry_list.unwrap();
        for entry in entry_list {
            if entry.is_empty() {
                continue;
            }
            println!("entry_name: {}, entry type: {:?}", entry.get_name(), entry.file_type as FileType);
        }
    }


}
