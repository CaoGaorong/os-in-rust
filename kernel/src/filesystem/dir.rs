use os_in_rust_common::{ASSERT, MY_PANIC};


use super::{dir_entry::{self, DirEntry, FileType}, fs::{self, FileSystem}, inode::OpenedInode};

/**
 * 文件系统中的目录的结构以及操作
 */



#[inline(never)]
pub fn init_root_dir() {
    // printkln!("init root dir");
    let file_system = fs::get_filesystem();
    if file_system.is_none() {
        MY_PANIC!("file system is not loaded");
    }
    let file_system = file_system.unwrap();
    file_system.load_root_dir();
}

/**
 * 目录的结构。位于内存的逻辑结构
 */
pub struct Dir<'a> {
    pub inode: &'a mut OpenedInode,
}

impl <'a>Dir<'a> {
    pub fn new(inode: &'a mut OpenedInode) -> Self {
        Self {
            inode,
        }
    }
    pub fn get_inode_ref(&mut self) -> &mut OpenedInode {
        self.inode
    }
}



#[inline(never)]
pub fn search(file_path: &str) -> Option<DirEntry> {
    let fs  = fs::get_filesystem();
    ASSERT!(fs.is_some());
    let result = dir_entry::search_dir_entry(fs.unwrap(), file_path);
    if result.is_none() {
        return Option::None;
    }
    Option::Some(result.unwrap().0)
}


#[derive(Debug)]
pub enum CreateDirError {
    DirEntryExist,
    CouldNotCreateRootDir,
    DirPathMustStartWithRoot,
}

/**
 * 创建一个目录
 */
pub fn mkdir_p(fs: &mut FileSystem, dir_path: &str) -> Result<bool, CreateDirError>{
    let dir_path = dir_path.trim();
    if "/".eq(dir_path) {
        return Result::Err(CreateDirError::CouldNotCreateRootDir);
    }
    if !dir_path.starts_with("/") {
        return Result::Err(CreateDirError::DirPathMustStartWithRoot);
    }
    let root_dir = fs.get_root_dir();

    let mut base_inode = root_dir.get_inode_ref();

    let mut dir_entry_split = dir_path.split("/");
    // 遍历每个entry
    while let Option::Some(entry_name) = dir_entry_split.next() {
        if entry_name.is_empty() {
            continue;
        }
        let mut base_dir = Dir::new(base_inode);
        // 创建子目录
        let sub_dir_entry = self::mkdir(fs, &mut base_dir, entry_name)?;
        // 基于子目录
        base_inode = fs.inode_open(sub_dir_entry.i_no);
    }
    return Result::Ok(true);
}

/**
 * 在parent_dir目录下，创建一个名为dir_name的子目录
 */
#[inline(never)]
pub fn mkdir(fs: &mut FileSystem, parent_dir: &mut Dir, dir_name: &str) -> Result<DirEntry, CreateDirError> {
    // 在该目录下创建一个文件夹类型的目录项
    let create_result = dir_entry::create_dir_entry(fs, parent_dir.inode, dir_name, FileType::Directory);
    let (dir_entry, _) = create_result.unwrap();
    // 该目录项下应该还有两项，分别是: ..和.
    let cur_inode = fs.inode_open(dir_entry.i_no);
    // 创建..目录项
    dir_entry::do_create_dir_entry(fs, cur_inode, Option::Some(parent_dir.inode.i_no), "..", FileType::Directory);
    // 创建 .目录项
    dir_entry::do_create_dir_entry(fs, cur_inode, Option::Some(dir_entry.i_no), ".", FileType::Directory);
    
    return Result::Ok(dir_entry);
}


#[inline(never)]
pub fn mkdir_in_root(dir_name: &str) -> Result<DirEntry, CreateDirError> {
    let file_system = fs::get_filesystem();
    if file_system.is_none() {
        MY_PANIC!("file system is not loaded");
    }
    let file_system = file_system.unwrap();
    let root_dir = file_system.get_root_dir();
    mkdir(file_system, root_dir, dir_name)
}

#[inline(never)]
pub fn mkdir_p_in_root(dir_path: &str) -> Result<bool, CreateDirError> {
    let file_system = fs::get_filesystem();
    if file_system.is_none() {
        MY_PANIC!("file system is not loaded");
    }
    let file_system = file_system.unwrap();
    mkdir_p(file_system, dir_path)
}


