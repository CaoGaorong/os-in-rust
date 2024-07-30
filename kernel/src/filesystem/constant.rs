use core::mem::size_of;

use os_in_rust_common::{constants, domain::InodeNo};

use super::dir_entry::DirEntry;

/**
 * 文件系统魔数
 */
pub const FILESYSTEM_MAGIC: u32 = 0x20010217;
/**
 * inode直接块的数据扇区数量
 */
pub const INODE_DIRECT_DATA_SECS: usize = 12;
/**
 * inode间接 数据扇区数量
 */
pub const INODE_INDIRECT_DATA_SECS: usize = 1;
/**
 * inode文件数据占用的扇区数量
 */
pub const INODE_DATA_SECS: usize = INODE_DIRECT_DATA_SECS + INODE_INDIRECT_DATA_SECS;

/**
 * 一个文件系统最大的文件数量（inode数量）
 */
pub const MAX_FILE_PER_FS: u32 = 4096;


/**
 * 文件名称最大长度。单位字节
 */
pub const MAX_FILE_NAME: usize = 20;

/**
 * 整个系统最大可以打开的文件数量
 */
pub const MAX_OPENED_FILE_IN_SYSTEM: usize = 32;

/**
 * 文件路径最大长度
 */
pub const MAX_FILE_PATH_LEN: usize = 100;


/**
 * 一个块里面最多有多少个目录项
 */
pub const MAX_ENTRY_IN_BLOCK: usize = constants::DISK_SECTOR_SIZE / size_of::<DirEntry>();