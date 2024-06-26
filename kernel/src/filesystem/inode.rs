use super::constant;

/**
 * inode的元信息。存储在硬盘的物理结构
 */
#[repr(C, packed)]
pub struct Inode {
    /**
     * inode编号
     */
    pub i_no: u32,
    /**
     * 当前inode占用的空间大小。单位：字节
     * inode是文件，那么i_size是文件大小
     * inode是目录，那么i_size是该目录下所有目录项的大小（不递归）
     */
    pub i_size: u32,

    /**
     * 该inode数据扇区所在的LBA地址。
     * 文件的数据内容分布在不同的扇区。i_sectors[0] = 112。位于第112扇区
     * 12个直接块 + 1个间接块
     */
    pub i_sectors: [Option<u32>; constant::INODE_DATA_SECS],
}
