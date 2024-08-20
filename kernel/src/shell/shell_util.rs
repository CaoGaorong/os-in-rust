
use os_in_rust_common::{array_deque::ArrayDeque, cstring_utils};
use crate::println;

#[derive(Debug)]
pub enum PathError {
    CwdNotStartWithRoot,
    AbsPathNotLongEnough,

}

/**
 * 已知当前工作目录cwd和输入的路径input_path，得到绝对路径 abs_path
 */
#[inline(never)]
pub fn get_abs_path<'a>(cwd: &str, input_path: &str, buff: &'a mut [u8]) -> Result<&'a str, PathError> {
    unsafe { buff.as_mut_ptr().write_bytes(0, buff.len()) };

    let cwd = cwd.trim();
    let input_path = input_path.trim();

    if !cwd.starts_with("/") {
        return Result::Err(PathError::CwdNotStartWithRoot);
    }
    if buff.len() <= cwd.len() + input_path.len() {
        return Result::Err(PathError::AbsPathNotLongEnough);
    }

    // 构建一个双端队列
    let mut deque: ArrayDeque<Option<&str>, 50> = ArrayDeque::new([Option::None; 50]);

    // 如果输入的不是/开头，那么说明输入的是相对路径，那么就需要处理当前工作目录
    if !input_path.starts_with("/") {
        let mut cwd_split = cwd.split("/");
        while let Option::Some(entry) = cwd_split.next() {
            if entry.is_empty() {
                continue;
            }
            // 如果是要上一步，那么弹出一层路径
            if entry.starts_with("..") {
                deque.pop_last();
                continue;
            }
            // 如果是当前路径，不变
            if entry == "." {
                continue;
            }
            deque.append(Option::Some(entry));
        }
    }

    // 处理输入的路径
    let mut input_split = input_path.split("/");
    while let Option::Some(entry) = input_split.next() {
        if entry.is_empty() {
            continue;
        }
        // 如果是要上一步，那么弹出一层路径
        if entry.starts_with("..") {
            deque.pop_last();
            continue;
        }
        // 如果是当前路径，不变
        if entry == "." {
            continue;
        }
        deque.append(Option::Some(entry));
    }

    // 绝对路径从/开始
    buff[..1].copy_from_slice("/".as_bytes());
    
    // 遍历所有的entry，构建绝对路径
    let mut idx = 1;
    for &entry in deque.get_array() {
        if entry.is_none() {
            continue;
        }
        let entry = entry.unwrap();
        buff[idx..idx + entry.len()].copy_from_slice(entry.as_bytes());
        idx += entry.len();
        buff[idx..idx + 1].copy_from_slice("/".as_bytes());
        idx += 1;
    };
    let abs_path = cstring_utils::read_from_bytes(buff);

    return Result::Ok(abs_path.unwrap());
}


