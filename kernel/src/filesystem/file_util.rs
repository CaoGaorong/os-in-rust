/**
 * 把一个文件全路径，分为父目录路径和当前文件的目录项名称
 */
pub fn split_file_path(path: &str) -> Option<(&str, &str)> {
    let mut path = path;
    if !path.starts_with("/") {
        return Option::None;
    }
    if path == "/" {
        return Option::None;
    }
    let mut last_slash_idx = path.rfind("/")?;

    // 去掉最后一个斜线
    if last_slash_idx == path.len() - 1 {
        path = &path[..path.len() - 1];
        last_slash_idx = path.rfind("/")?;
    }
     // 该文件的目录路径, 该文件的名称
     let (dir_path, file_name) = if last_slash_idx == 0 {
        ("/", &path[1..])
    } else {
        (&path[..last_slash_idx],  &path[last_slash_idx+1..])
    };
    return Option::Some((dir_path, file_name));
}


// #[inline(never)]
pub fn reverse_path(src: &str, separator: &str, dest: &mut [u8]) {
    let src = src.trim();
    assert!(dest.len() >= src.len());
    // 目标字符串
    let dest = &mut dest[..src.len()];

    let mut src_idx = 0;
    let mut dest_idx = dest.len();
    
    // 开头是否有分隔符
    let start_sep = src.starts_with(separator);
    // 结尾是否有分隔符
    let end_sep = src.ends_with(separator);
    // 对于源字符串，跳过开头的分隔符
    if start_sep {
        src_idx += separator.len();
    }
    // 对于目标字符串，预留结尾的分隔符
    if end_sep {
        dest_idx -= separator.len();
    }

    // 使用分隔符分隔
    let mut split = src.split(separator);
    let len = dest.len();
    // 遍历每一项
    while let Option::Some(item) = split.next() {
        if item.is_empty() {
            continue;
        }
        // 把遍历到的项 复制到 目标字符串
        let from  = (&src[src_idx .. src_idx + item.len()]).as_bytes();
        let to = &mut dest[dest_idx - item.len() .. dest_idx];
        assert_eq!(from.len(), to.len());
        to.copy_from_slice(from);
        src_idx += item.len();
        dest_idx -= item.len();

        if dest_idx < separator.len() {
            continue;
        }
        // 再把这次的分隔符 复制到 目标项
        let to = &mut dest[dest_idx - separator.len() .. dest_idx];
        to.copy_from_slice(separator.as_bytes());
        src_idx += separator.len();
        dest_idx -= separator.len();
    }

    // 如果原本开头有分隔符，那么也要加上
    if start_sep {
        let to = &mut dest[ .. separator.len()];
        to.copy_from_slice(separator.as_bytes());
    }
    // 如果结尾有分隔符
    if end_sep {
        let to = &mut dest[len - separator.len() .. ];
        to.copy_from_slice(separator.as_bytes());
    }

}

pub mod test {
    use crate::filesystem::file_util::reverse_path;

    use super::split_file_path;

    #[test]
    fn test_split_util() {
        println!("{:?}", split_file_path("/"));
        println!("{:?}", split_file_path("/dev/"));
        println!("{:?}", split_file_path("/dev/proc/"));
        println!("{:?}", split_file_path("/a.txt"));
    }
    #[test]
    pub fn test_split_inclusive() {
        let src = "dev/proc/test";
        let mut split = src.split("/");
        while let Some(item) = split.next() {
            
            println!("{:?}", item);
        }
    }

    #[test]
    pub fn test_reverse_str() {
        let src = "/";
        let mut buf = [0u8; 100];
        reverse_path(src, "/", &mut buf);
        let dest = core::str::from_utf8(&mut buf).unwrap();
        println!("src :{}, len:{}", src, src.len());
        println!("dest:{}, len:{}", dest, dest.len());
    }
}