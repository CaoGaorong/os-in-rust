use super::{cmd_custom, cmd_dir, cmd_file};
use super::{cmd::Cmd, cmd_cd, cmd_ls, cmd_ps};

use crate::{print, println};
use crate::sys_call;

#[inline(never)]
pub fn execute_cmd(cwd: &str, cmd: Cmd, param: Option<&str>, buf: &mut [u8]) {
    // 清空缓冲区
    unsafe { buf.as_mut_ptr().write_bytes(0, buf.len()) };
    match cmd { 
        Cmd::Pwd => {
            print!("{}", cwd);
        },
        Cmd::Ps => {
            cmd_ps::ps();
        },
        Cmd::Ls => {
            cmd_ls::ls(cwd, param, buf);
        },
        Cmd::Cd => {
            let res = cmd_cd::cd(cwd, param, buf);
            if res.is_some() {
                // println!("change directory to {}", res.unwrap());
            } else {
                println!("cd error {} not exist", param.unwrap());
            }
        },
        // 清屏
        Cmd::Clear => {
            sys_call::clear_screen();
        },
        // 创建目录
        Cmd::Mkdir => {
            cmd_dir::mkdir(cwd, param, buf);
        },
        // 删除目录
        Cmd::Rmdir => {
            cmd_dir::rmdir(cwd, param, buf);
        },
        // 创建普通文件
        Cmd::Touch => {
            cmd_file::create_file(cwd, param, buf);
        },
        Cmd::Rm => {
            cmd_file::remove_file(cwd, param, buf);
        },
        Cmd::Custom(cmd) => {
            cmd_custom::custom_cmd(cwd, cmd, param, buf);
        },
    };
}