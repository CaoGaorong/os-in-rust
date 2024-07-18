use core::{fmt::{Display, Error}, mem::{self, size_of}, ops::Index, ptr, slice};

use os_in_rust_common::{bitmap::BitMap, constants, cstr_write, cstring_utils::{self, sprintf_fn}, domain::{InodeNo, LbaAddr}, linked_list::LinkedList, printkln, racy_cell::RacyCell, utils, ASSERT, MY_PANIC};

use crate::{filesystem::{file::OpenedFile, global_file_table}, memory, thread};

use super::{constant, fs::{self, FileSystem}, inode::{Inode, InodeLocation, OpenedInode}, superblock::SuperBlock};

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
 * 文件的类型
 */
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum FileType {
    /**
     * 普通文件
     */
    Regular,
    /**
     * 目录
     */
    Directory,
    /**
     * 未知
     */
    Unknown,
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
/**
 * 目录项的结构。物理结构，保存到硬盘中
 */
#[derive(Debug)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DirEntry {
    /**
     * 该目录项对应的inode编号
     */
    pub i_no: InodeNo, 
    /**
     * 目录项名称
     */
    name:  [u8; constant::MAX_FILE_NAME],
    /**
     * 文件类型
     */
    file_type: FileType,
}

impl DirEntry {
    pub fn new(i_no: InodeNo, file_name: &str, file_type: FileType) -> Self {
        let mut dir_entry = Self {
            i_no: i_no,
            name: [0; constant::MAX_FILE_NAME],
            file_type: file_type,
        };
        // 写入文件名称
        cstr_write!(&mut dir_entry.name, "{}", file_name);
        dir_entry
    }

    #[inline(never)]
    pub fn get_name(&self) -> &str {
        let name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(name.is_some());
        name.unwrap()
    }

    pub fn is_empty(&self) -> bool {
        let i = usize::from(self.i_no);
        // 这是根目录，非空
        if i == 0 && self.file_type as FileType  == FileType::Directory {
            return false;
        }
        return i == 0;
    }
}


#[derive(Debug)]
pub enum FindFileError {
    FileNotFound
}

#[inline(never)]
pub fn search(file_path: &str) -> Result<DirEntry, FindFileError> {
    let fs  = fs::get_filesystem();
    ASSERT!(fs.is_some());
    search_file(fs.unwrap(), file_path)
}

#[inline(never)]
pub fn search_file(filesystem: &mut FileSystem, file_path: &str) -> Result<DirEntry, FindFileError> {

    let root_inode_no = filesystem.get_root_dir().inode.i_no;
    
    // 如果就是根目录
    if file_path == "/" {
        return Result::Ok(DirEntry::new(root_inode_no, file_path, FileType::Directory));
    }

    // 成功搜索过的路径
    let mut searched_path = "/";
    // 当前的inode，是根目录的inode
    let mut cur_inode = filesystem.inode_open(root_inode_no);
    // 当前的目录项。默认是根目录
    let mut cur_dir_entry = DirEntry::new(root_inode_no, "/", FileType::Directory);

    // 把要搜索的路径，分隔
    let mut file_entry_split = file_path.split("/");
    while let Option::Some(file_entry_name) = file_entry_split.next() {
        if file_entry_name.is_empty() {
            continue;
        }
        // 根据名称搜索目录项
        let dir_entry = search_dir_entry(filesystem, &mut cur_inode, file_entry_name);
        // 如果目录项不存在
        if dir_entry.is_none() {
            return Result::Err(FindFileError::FileNotFound);
        }

        let dir_entry = dir_entry.unwrap();
        // 根据inode号，打开
        let opened_inode = filesystem.inode_open(dir_entry.i_no);
        cur_inode = opened_inode;

        // 搜索下一个目录项
        cur_dir_entry = dir_entry;

        // 搜索过的路径
        // searched_path = searched_path + "/" +  dir_entry.get_name();
    }
    // 返回找到的最后的那个目录项
    return Result::Ok(cur_dir_entry);
}


/**
 * 查找某个目录下，名为entry_name的目录项
 */
#[inline(never)]
fn search_dir_entry(fs: &mut FileSystem, dir_inode: &mut OpenedInode, entry_name: &str) -> Option<DirEntry> {
    // 如果直接块都满了，那么就需要加载间接块
    if dir_inode.get_direct_data_blocks_ref().iter().all(|block| !block.is_empty()) {
        dir_inode.load_data_block(fs);
    }

    // 取出所有的数据块
    let data_blocks = dir_inode.get_data_blocks_ref();

    let disk = unsafe { &mut *fs.base_part.from_disk };
    
    // 开辟缓冲区
    let buff_addr = memory::sys_malloc(constants::DISK_SECTOR_SIZE);
    let buff_u8 = unsafe { slice::from_raw_parts_mut(buff_addr as *mut u8, constants::DISK_SECTOR_SIZE) };

    // 遍历所有的数据区，根据名称找目录项
    for block_lba in data_blocks {
        if block_lba.is_empty() {
            continue;
        }
        disk.read_sectors(*block_lba, 1, buff_u8);

        // 读取出的数据，转成页目录项列表
        let dir_entry_list = unsafe { slice::from_raw_parts(buff_addr as *const DirEntry, constants::DISK_SECTOR_SIZE / size_of::<DirEntry>()) };
        let find = dir_entry_list.iter().find(|entry| entry.get_name() == entry_name);
        
        // 找到了，直接返回
        if find.is_some() {
            return Option::Some(*find.unwrap());
        }
    }
    return Option::None;
}


#[derive(Debug)]
pub enum CreateDirError {
    DirEntryExist,
    CouldNotCreateRootDir,
    DirPathMustStartWithRoot,
}

/**
 * 在filesystem文件系统中，parent_dir目录下创建一个名为dir_name的目录项
 * 在实现上，分成两个步骤：
 *   - 创建文件（inode）以及对应的目录项（文件名称）
 *   - 把这个inode挂到该目录下（把目录项放写入到目录对应的数据区）
 */
#[inline(never)]
pub fn create_dir_entry(fs: &mut FileSystem, parent_dir: &mut Dir, entry_name: &str, file_type: FileType) -> Result<DirEntry, CreateDirError> {

    /****1. 创建一个目录项 */
    let (created_dir_entry, created_entry_inode) = self::do_create_dir_entry(fs, parent_dir, Option::None, entry_name, file_type)?;

    /***2. 填充内存结构*****/
    // 2.1 添加到打开的分区中
    fs.open_inodes.append(&mut created_entry_inode.tag);

    // 2.2 在整个系统打开的文件中，注册一下
    let file_table_idx = global_file_table::register_file(OpenedFile::new(created_entry_inode));
    ASSERT!(file_table_idx.is_some());
    let file_table_idx = file_table_idx.unwrap();

    // 2.3 把这个打开的文件，安装到当前进程的文件描述符
    thread::current_thread().task_struct.fd_table.install_fd(file_table_idx);

    // 返回创建好了目录项
    Result::Ok(created_dir_entry)
}

/**
 * 在parent_dir目录下，创建名为entry_name，并且inode号为entry_inode的目录项。
 */
#[inline(never)]
pub fn do_create_dir_entry(fs: &mut FileSystem, parent_dir: &mut Dir, entry_inode: Option<InodeNo>, entry_name: &str, file_type: FileType) -> Result<(DirEntry, &'static mut OpenedInode), CreateDirError> {
    let (inode_no, opened_inode) = if entry_inode.is_none() {
        /***1. 创建文件的inode。物理结构，同步到硬盘中*****/
        // 从当前分区中，申请1个inode，并且写入硬盘（inode位图）
        let inode_no = fs.inode_pool.apply_inode(1);

        // 创建一个inode
        let inode = Inode::new(inode_no);
        let opened_inode: &mut OpenedInode = memory::malloc(size_of::<OpenedInode>());
        *opened_inode = OpenedInode::new(inode);

        // 把inode写入硬盘（inode列表）
        opened_inode.sync_inode(fs);
        (inode_no, opened_inode)
    } else {
        let inode_no = entry_inode.unwrap();
        let opened_inode = fs.inode_open(inode_no);
        (inode_no, opened_inode)
    };

    /***2. 把这个新文件作为一个目录项，挂到父目录中*****/
    // 创建一个目录项
    let dir_entry = DirEntry::new(inode_no, entry_name, file_type);
    let search_result = self::search_dir_entry(fs, parent_dir.inode, entry_name);
    // 如果这个目录项，已经存在同名的了
    if search_result.is_some() {
        return Result::Err(CreateDirError::DirEntryExist);
    }

    // 把目录项挂到目录并且写入硬盘（inode数据区）
    fs.sync_dir_entry(parent_dir, &dir_entry);
    
    return Result::Ok((dir_entry, opened_inode));
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
    let dir_entry = self::create_dir_entry(fs, parent_dir, dir_name, FileType::Directory)?;
    // 该目录项下应该还有两项，分别是: ..和.
    let mut cur_dir = Dir::new(fs.inode_open(dir_entry.i_no));
    // 创建..目录项
    self::do_create_dir_entry(fs, &mut cur_dir, Option::Some(parent_dir.inode.i_no), "..", FileType::Directory)?;
    // 创建 .目录项
    self::do_create_dir_entry(fs, &mut cur_dir, Option::Some(dir_entry.i_no), ".", FileType::Directory)?;
    
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


