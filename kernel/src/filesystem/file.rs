use core::mem::size_of;

use os_in_rust_common::{constants, domain::InodeNo, racy_cell::RacyCell, ASSERT};

use crate::{filesystem::{dir::{DirEntry, FileType}, global_file_table}, memory, thread};

use super::{dir::Dir, fs::FileSystem, inode::{Inode, OpenedInode}};

/**
 * 文件系统的，文件的结构
 */
pub struct OpenedFile {
    /**
     * 这个打开的文件，底层指向的inode节点
     */
    inode: &'static OpenedInode,
    /**
     * 操作的文件的偏移量（单位字节）
     */
    file_off: usize,
    /**
     * 这个打开的文件的操作标识
     */
    flag: FileFlag,
}

impl OpenedFile {
    pub fn new(inode: &'static OpenedInode) -> Self {
        Self {
            inode,
            file_off: 0,
            flag: FileFlag::Init,
        }
    }
}
pub enum FileFlag {
    Init,
}

/**
 * 标准文件描述符
 */
pub enum StdFileDescriptor {
    /**
     * 标准输入
     */
    StdInputNo = 0x0,
    /**
     * 标准输出
     */
    StdOutputNo = 0x1,
    /**
     * 标准错误
     */
    StdErrorNo = 0x2,
}

/**
 * 在part分区中，parent_dir目录下创建一个名为file_name的文件
 * 在实现上，分成两个步骤：
 *   - 创建文件（inode）以及对应的目录项（文件名称）
 *   - 把这个inode挂到该目录下（把目录项放写入到目录对应的数据区）
 */
#[inline(never)]
pub fn create_file(filesystem: &mut FileSystem, parent_dir: &mut Dir, file_name: &str) {

    /***1. 创建文件的inode。物理结构，同步到硬盘中*****/
    // 从当前分区中，申请1个inode，并且写入硬盘（inode位图）
    let inode_no = filesystem.inode_pool.apply_inode(1);

    // 创建一个inode
    let inode = Inode::new(inode_no);
    let opened_inode: &mut OpenedInode = memory::malloc(size_of::<OpenedInode>());
    *opened_inode = OpenedInode::new(inode);

    // 把inode写入硬盘（inode列表）
    opened_inode.sync_inode(filesystem);

    /***2. 把这个新文件作为一个目录项，挂到父目录中*****/
    // 创建一个目录项
    let dir_entry = DirEntry::new(inode_no, file_name, FileType::Regular);
    // 把目录项挂到目录并且写入硬盘（inode数据区）
    filesystem.sync_dir_entry(parent_dir, &dir_entry);


    /***3. 填充内存结构*****/
    // 3.1 添加到打开的分区中
    filesystem.open_inodes.append(&mut opened_inode.tag);

    // 3.2 在整个系统打开的文件中，注册一下
    let file_table_idx = global_file_table::register_file(OpenedFile::new(opened_inode));
    ASSERT!(file_table_idx.is_some());
    let file_table_idx = file_table_idx.unwrap();

    // 3.3 把这个打开的文件，安装到当前进程的文件描述符
    thread::current_thread().task_struct.fd_table.install_fd(file_table_idx);
}



