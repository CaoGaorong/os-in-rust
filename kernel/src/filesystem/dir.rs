use os_in_rust_common::domain::InodeNo;
use os_in_rust_common::{instruction, printkln, ASSERT, MY_PANIC};
use crate::filesystem::dir_entry::DirEntrySearchReq;
use crate::{memory, thread};
use crate::thread::TaskStruct;

use super::{dir_entry::{self, FileType}, file, file_util, fs::{self, FileSystem}, inode::{self, OpenedInode}};

/** 
 * 文件系统中的目录的结构以及操作
 */



#[inline(never)]
pub fn init_root_dir() {
    // 设置文件小系统的根目录
    self::load_root_dir();

    // 每个任务的当前工作目录都设置为根目录
    self::set_root_dir_for_task();
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


#[inline(never)]
fn set_root_dir_for_task() {
    let old = instruction::disable_interrupt();
    let file_system = fs::get_filesystem();
    for tag in thread::get_all_thread().iter() {
        let task = unsafe { &mut *TaskStruct::parse_by_all_tag(&*tag) };
        task.cwd_inode = Option::Some(file_system.super_block.root_inode_no);
    }
    instruction::set_interrupt(old);
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


/**
 * 得到这个任务的工作路径
 */
#[inline(never)]
pub fn get_cwd<'a>(task: &TaskStruct, path: &'a mut [u8]) -> Option<&'a str> {
    if task.cwd_inode.is_none() {
        return Option::None;
    }

    let buf: &mut [u8; 100] = memory::malloc(100);
    let fs = fs::get_filesystem();
    let mut base_inode = inode::inode_open(fs, task.cwd_inode.unwrap());
    let mut idx = 0;
    loop {
        // 找到这个inode对应的父目录的inode
        let parent_inode = dir_entry::parent_entry(base_inode);

        // 当前目录跟父目录的inode号相同，说明是根目录直接返回
        if base_inode.i_no == parent_inode {
            break;
        }

        // 打开父目录
        let parent_inode = inode::inode_open(fs, parent_inode);
        // 父目录下搜索当前目录，得到目录项名称
        let cur_dir_entry = dir_entry::do_search_dir_entry(fs, parent_inode, DirEntrySearchReq::build().i_no(base_inode.i_no));
        inode::inode_close(fs, base_inode);

        // 当前节点，对应不上目录项。也返回
        if cur_dir_entry.is_none() || cur_dir_entry.unwrap().is_empty() {
            inode::inode_close(fs, parent_inode);
            break;
        }
        let cur_dir_entry = cur_dir_entry.unwrap();
        // 拼接目录项名称
        let entry_name = cur_dir_entry.get_name();
        // 把这个目录名称保存下来
        buf[idx ..idx + 1].copy_from_slice("/".as_bytes());
        buf[idx + 1 ..idx + 1 + entry_name.len()].copy_from_slice(entry_name.as_bytes());
        idx += "/".len();
        idx += entry_name.len();
        base_inode = parent_inode;
    }
    if idx == 0 {
        buf[idx ..idx + 1].copy_from_slice("/".as_bytes());
        idx += "/".len();
    }

    let min_len = path.len().min(buf.len()).min(idx);
    let from_path = core::str::from_utf8(&buf[.. min_len]);
    if from_path.is_err() {
        MY_PANIC!("failed to get work directory. error:{:?}", from_path);
    }
    let from_path = from_path.unwrap();

    file_util::reverse_path(&from_path[..min_len], "/", &mut path[..min_len]);

    memory::sys_free(buf.as_ptr() as usize);
    
    let path = core::str::from_utf8(&path[..min_len]);
    Option::Some(path.unwrap())
}


/**
 * 切换这个任务的工作目录
 */
#[inline(never)]
pub fn change_dir(task: &mut TaskStruct, path: &str) -> Option<()> {
    let fs = fs::get_filesystem();
    // 找到这个目录项
    let (entry, entry_inode) = dir_entry::search_dir_entry(fs, path)?;
    // 关闭inode
    inode::inode_close(fs, entry_inode);
    // 取出inode号
    task.cwd_inode = Option::Some(entry.i_no);

    return Option::Some(());
}