use crate::sys_call;

use super::shell_util;


#[inline(never)]
pub fn cd<'a>(cwd: &str, param: Option<&str>, buf: &'a mut [u8]) -> Option<&'a str> {
    let abs_path = if param.is_none() {
        shell_util::get_abs_path(cwd, "/", buf).unwrap()
    } else {
        let abs_path = shell_util::get_abs_path(cwd, param.unwrap(), buf).unwrap();
        abs_path
    };
    let res = sys_call::read_dir(abs_path);
    if res.is_err() {
        return Option::None;
    }
    // 系统调用，切换当前任务的工作目录
    sys_call::change_dir(abs_path)?;

    return Option::Some(abs_path);
}