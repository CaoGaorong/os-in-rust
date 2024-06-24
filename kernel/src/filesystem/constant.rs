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