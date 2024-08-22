use crate::sys_call;

use super::shell_util;
use crate::println;

#[inline(never)]
pub fn exec(cwd: &str, cmd: &str, buff: &mut [u8]) {
    let cmd_path = shell_util::get_abs_path(cwd, cmd, buff);
    if cmd_path.is_err() {
        println!("failed to get abs path, cwd:{}, cmd:{}, error:{:?}", cwd, cmd, cmd_path.unwrap_err());
        return;
    }
    
    let cmd_path = cmd_path.unwrap();
    
    // 执行exec系统调用
    let exe_res = sys_call::exec(cmd_path);
    if exe_res.is_err() {
        println!("failed to exec {}, error:{:?}", cmd, exe_res.unwrap_err());
        return;
    }
}