use os_in_rust_common::domain::InodeNo;

use super::{dir_entry::{self, FileType}, file, fs::{self, FileSystem}, inode::{self, OpenedInode}};

/** 
 * 文件系统中的目录的结构以及操作
 */



#[inline(never)]
pub fn init_root_dir() {
    self::load_root_dir();
}

/**
 * 加载根目录。
 */
#[inline(never)]
pub fn load_root_dir() {
    let file_system = fs::get_filesystem();
    let root_inode = inode::load_inode(file_system, file_system.super_block.root_inode_no);
    file_system.set_root_inode(root_inode);
}

/**
 * 在parent_dir目录下，创建一个名为dir_name的子目录
 */
#[inline(never)]
pub fn mkdir(fs: &mut FileSystem, parent_dir_inode: &mut OpenedInode, dir_name: &str) -> InodeNo {
    let parent_ino = parent_dir_inode.i_no;
    // 在该目录下创建一个文件夹类型的目录项
    let entry_i_no = dir_entry::create_dir_entry(fs, parent_dir_inode, dir_name, FileType::Directory);
    let entry_inode = inode::inode_open(fs, entry_i_no);
    // 该目录项下应该还有两项，分别是: ..和.
    // 创建..目录项
    dir_entry::do_create_dir_entry_with_inode(fs, entry_inode, parent_ino, "..", FileType::Directory);
    // 创建 .目录项
    dir_entry::do_create_dir_entry_with_inode(fs, entry_inode, entry_inode.i_no, ".", FileType::Directory);

    inode::inode_close(fs, entry_inode);

    entry_i_no
}


