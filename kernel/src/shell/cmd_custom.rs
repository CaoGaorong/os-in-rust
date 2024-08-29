use crate::{common::exec_dto::ExecParam, sys_call};

use super::shell_util;
use crate::println;

#[inline(never)]
pub fn custom_cmd<'a> (cwd: &str, cmd: &str, param: Option<&str>, buf: &'a mut [u8]) {
    let fork_res = sys_call::fork();
    match fork_res {
        sys_call::ForkResult::Parent(child_id) => {
            // 阻塞等待子进程退出
            let wait_res = sys_call::wait();
            if wait_res.is_none() {
                println!("child process does not exit");
            }
            let (chpid, exit_status) = wait_res.unwrap();
            println!("child process exit. fork child pid:{}, child pid:{}, child status:{:?}", child_id.get_data(), chpid.get_data(), exit_status);
        },
        sys_call::ForkResult::Child => {
            self::exec(cwd, cmd, param, buf);
        },
    }
}

fn cmd_dispatch<'a>(cwd: &str, cmd: &str, param: Option<&str>, buf: &'a mut [u8]) {
    if param.is_none() {
        self::exec(cwd, cmd, Option::None, buf);
        return;
    }
    let param = param.unwrap();
    // 把参数，按照管道分隔
    let pipe_split = param.split("|");
    // 管道的数量
    let pipe_cnt = pipe_split.count();

    
}


#[inline(never)]
fn exec(cwd: &str, cmd: &str, param: Option<&str>, buff: &mut [u8]) {
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