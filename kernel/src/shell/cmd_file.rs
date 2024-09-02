use crate::{filesystem, println, sys_call};

use super::shell_util;

#[inline(never)]
pub fn create_file(cwd: &str, param: Option<&str>, buff: &mut [u8]) {
    if param.is_none() || param.unwrap().is_empty() {
        println!("please input file name");
        return;
    }
    let file_name = param.unwrap();
    let file_path = shell_util::get_abs_path(cwd, file_name, buff);
    if file_path.is_err() {
        println!("failed to create file {}, error:{:?}", file_name, file_path.unwrap_err());
        return;
    }
    let file_path = file_path.unwrap();
    let file = sys_call::File::create(file_path);
    if file.is_err() {
        println!("failed to create file {}, error:{:?}", file_name, file.unwrap_err());
        return;
    }
}

#[inline(never)]
pub fn remove_file(cwd: &str, param: Option<&str>, buff: &mut [u8]) {
    if param.is_none() || param.unwrap().is_empty() {
        println!("please input file name");
        return;
    }
    let file_name = param.unwrap();
    let file_path = shell_util::get_abs_path(cwd, file_name, buff);
    if file_path.is_err() {
        println!("failed to create file {}, error:{:?}", file_name, file_path.unwrap_err());
        return;
    }
    let remove_res = sys_call::remove_file(file_path.unwrap());
    if remove_res.is_err() {
        println!("failed to remove {}, error:{:?}", file_name, remove_res.unwrap_err());
    }
}