use os_in_rust_common::ASSERT;

use crate::filesystem;
use crate::{print};

/**
 * ls命令
 */
#[inline(never)]
pub fn ls(dir_path: &str, param: Option<&str>) {
    let dir = filesystem::read_dir(dir_path);
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
    }
}