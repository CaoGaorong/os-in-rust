
use os_in_rust_common::racy_cell::RacyCell;

/**
 * 用来保存用户进程运行的时候，传递的参数
 */
static ARGS: RacyCell<Option<&str>> = RacyCell::new(Option::None);

/**
 * 设置参数
 */
pub fn set_args(args: &'static str) {
    *unsafe { ARGS.get_mut() } = Option::Some(args);
}

/**
 * 获取参数
 */
pub fn get_args() -> Option<&'static str> {
    let args = unsafe { ARGS.get_mut() };
    args.as_deref()
}