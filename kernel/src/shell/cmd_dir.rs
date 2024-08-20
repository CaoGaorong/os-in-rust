
use crate::{filesystem, println, shell::shell_util, sys_call};

#[inline(never)]
pub fn mkdir(cwd: &str, param: Option<&str>, buf: &mut [u8]) {
    if param.is_none() {
        println!("please enter the director name.");
        return;
    }
    let param = param.unwrap().trim();
    if param.is_empty() {
        println!("please enter the director name.");
        return;
    }
    
    unsafe { buf.as_mut_ptr().write_bytes(0, buf.len()) };

    let param_split = param.split_once(" ");
    // 只有一个参数，那么就是mkdir directory
    if param_split.is_none() {
        let dir_path: &str = shell_util::get_abs_path(cwd, param, buf).unwrap();
        let create_res = sys_call::create_dir(dir_path);
        if create_res.is_err() {
            println!("{:?}", create_res.unwrap_err());
        }
        return;
    }
    // 如果有两个参数，那么可能是mkdir -p /test/srst
    let (arg, dir_name) = param_split.unwrap();
    if arg.trim() != "-p" {
        println!("mkdir not support with {}", arg);
        return;
    }

    let dir_path = shell_util::get_abs_path(cwd, dir_name, buf).unwrap();
    let create_res = sys_call::create_dir_all(dir_path);
    if create_res.is_err() {
        println!("{:?}", create_res.unwrap_err());
    }
}

#[inline(never)]
pub fn rmdir(cwd: &str, param: Option<&str>, buf: &mut [u8]) {
    unsafe { buf.as_mut_ptr().write_bytes(0, buf.len()) };
    
    if param.is_none() {
        println!("please enter the director name.");
        return;
    }
    let param = param.unwrap().trim();
    if param.is_empty() {
        println!("please enter the director name.");
        return;
    }

    let dir_path = shell_util::get_abs_path(cwd, param, buf).unwrap();

    let remove_res = sys_call::remove_dir(dir_path);
    match remove_res {
        Ok(()) => {
            return;
        },
        Err(err) => {
            match err {
                filesystem::DirError::DirPathIllegal => todo!(),
                filesystem::DirError::NotFound => {
                    println!("dir path not exist: {}", dir_path);
                    return;
                },
                filesystem::DirError::ParentDirNotExists => todo!(),
                filesystem::DirError::AlreadyExists => todo!(),
                filesystem::DirError::DirectoryNotEmpty => {
                    println!("dir not empty: {}", dir_path);
                    return;
                },
            }
        },
    }
}