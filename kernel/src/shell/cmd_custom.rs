use crate::{common::exec_dto::ExecParam, sys_call};

use super::shell_util;
use crate::println;

#[inline(never)]
pub fn exec(cwd: &str, cmd: &str, param: Option<&str>, buff: &mut [u8]) {
    let cmd_path = shell_util::get_abs_path(cwd, cmd, buff);
    if cmd_path.is_err() {
        println!("failed to get abs path, cwd:{}, cmd:{}, error:{:?}", cwd, cmd, cmd_path.unwrap_err());
        return;
    }
    
    let cmd_path = cmd_path.unwrap();
    let exec_param = ExecParam::new(cmd_path, param);
    
    // 执行exec系统调用
    let exe_res = sys_call::exec(&exec_param);
    if exe_res.is_err() {
        println!("failed to exec {}, error:{:?}", cmd, exe_res.unwrap_err());
        return;
    }
}