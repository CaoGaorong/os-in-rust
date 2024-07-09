#[cfg(test)]
mod tests {
    use std::{fs::File, io::{Read, Seek}, mem::size_of, slice};

    use kernel::filesystem::{dir::DirEntry, inode::{Inode, OpenedInode}};
    use os_in_rust_common::constants;

    #[test]
    fn test_opened_inode() {
        let opened_inode = OpenedInode::new(Inode::empty());
        println!("cache size:{}", opened_inode.data_block_list.len())
    }

    #[test]
    fn test_inode_size() {
        println!("inode size:{} ", size_of::<Inode>())
    }

    #[test]
    fn test_root_inode() {
        println!("dir size:{}", size_of::<DirEntry>());

        let hd80m = "build/hd80M.img";
        let mut file = File::open(hd80m).expect("failed to open hd80M.img");
        file.seek(std::io::SeekFrom::Start(100)).expect("failed to seek file");
        

        let mut arr = [0x00u8; size_of::<Inode>()];
        arr[4] = 0x38;
        arr[8] = 0xe0;
        arr[9] = 0xeb;
        let inode = unsafe { *(arr.as_ptr() as *const Inode) };
        println!("{:?}", inode);
    }


    /**
     * 根目录
     */
    #[test]
    fn list_root_dir_entry() {
        // 硬盘中的数据
        let mut arr = [0x0u8; constants::DISK_SECTOR_SIZE];
        arr[4] = 0x2e;
        arr[24] = 0x01;
        arr[32] = 0x2e;
        arr[33] = 0x2e;
        arr[52] = 0x01;

        // 转成目录项的结构
        let dir_list = unsafe { slice::from_raw_parts(arr.as_ptr() as *const DirEntry, arr.len() / size_of::<DirEntry>()) };

        // 打印一下，看下结果
        dir_list.iter().filter(|entry| entry.is_valid()).for_each(|e| {
            println!("{:?}, entry name: {}", e, e.get_name());
            println!("------");
        });
    }
}