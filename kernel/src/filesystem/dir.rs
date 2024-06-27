use super::constant;

/**
 * 文件系统中的目录的结构以及操作
 */


/**
 * 文件的类型
 */
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
pub struct Dir {

}
/**
 * 目录项的结构。物理结构，保存到硬盘中
 */
#[repr(C, packed)]
pub struct DirEntry {
    /**
     * 该目录项对应的inode编号
     */
    pub i_no: usize, 
    /**
     * 目录项名称
     */
    pub name:  [u8; constant::MAX_FILE_NAME],
    /**
     * 文件类型
     */
    pub file_type: FileType,
}