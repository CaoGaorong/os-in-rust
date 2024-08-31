use super::{cmd_custom, cmd_dir};
use super::{cmd::Cmd, cmd_cd, cmd_ls, cmd_ps, shell, shell_util};

use crate::filesystem::{FileDescriptor, StdFileDescriptor};
use crate::{print, println};
use crate::sys_call;

/** 
 * 一个命令的结构
*/
pub struct Command<'a> {
    /**
     * 命令执行时，当前工作目录
     */
    cwd: &'a str,
    /**
     * 命令
     */
    cmd: Cmd<'a>,
    /**
     * 命令所需要的参数
     */
    param: Option<&'a str>
}
/**
 * 命令执行器
 */
pub trait CommandExecutor {
    fn execute(cwd: &str, cmd: &str);
}

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
        Cmd::Custom(cmd) => {
            cmd_custom::custom_cmd(cwd, cmd, param, buf);
        },
    };
}