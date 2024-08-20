use os_in_rust_common::{cstr_write, cstring_utils, ASSERT};

use crate::filesystem::{self, inode};
use crate::shell::shell_util;
use crate::sys_call::{self};
use crate::{print, println};

/**
 * ls命令
 */
#[inline(never)]
pub fn ls(cwd: &str, param: Option<&str>, buff: &mut [u8]) {
    let dir = sys_call::read_dir(cwd);
    ASSERT!(dir.is_ok());
    let mut dir = dir.unwrap();

    // 如果ls没有任何参数，那么就是直接打印
    if param.is_none() {
        for dir_entry in dir.iter() {
            // 私有的目录项，不展示
            let entry_name = dir_entry.get_name();
            if entry_name.starts_with(".") {
                continue;
            }
            print!("{} ", entry_name);
        }
        return;
    }
    // 如果有参数
    let param = param.unwrap().trim();
    // 只支持 -l参数
    if param != "-l" {
        print!("invalid param: {} for ls", param);
        return;
    }

    println!("total: {}", dir.get_file_size());
    println!("file_type inode_no file_size file_name");
    
    // 如果是-l参数
    for dir_entry in dir.iter() {
        unsafe { buff.as_mut_ptr().write_bytes(0, buff.len()) };

        // 私有的目录项，不展示
        let entry_name = dir_entry.get_name();
        if entry_name.starts_with(".") {
            continue;
        }
        
        let file_type = &(dir_entry.file_type as filesystem::FileType);
        let file_type_sign = self::get_file_type_sign(file_type);
        let file_inode = dir_entry.i_no;

        let file_size = self::get_file_size(shell_util::get_abs_path(cwd, entry_name, buff).unwrap());

        println!("{:^9} {:^8} {:^9} {:^9}", file_type_sign, file_inode.get_data(), file_size, dir_entry.get_name());
    }
    return;
}


/**
 * 得到文件类型的标识：
 *  - 普通文件：使用"-"标识
 *  - 目录文件：使用"d"标识
 * 
 */
fn get_file_type_sign(ft: &filesystem::FileType) -> &str {
    match ft {
        filesystem::FileType::Regular => "-",
        filesystem::FileType::Directory => "d",
        filesystem::FileType::Unknown => "*",
    }
}


/**
 * 得到文件的大小
 */
#[inline(never)]
fn get_file_size(file_path: &str) -> usize {
    let file = sys_call::File::open(file_path);
    if file.is_err() {
        println!("{:?}", file.unwrap_err());
        return 0;
    }
    let file = file.unwrap();
    let file_size = file.get_size();
    if file_size.is_err() {
        println!("{:?}", file_size.unwrap_err());
        return 0;
    }
    file_size.unwrap()
}