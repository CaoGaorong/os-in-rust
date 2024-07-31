
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

pub mod test {
    use super::split_file_path;

    #[test]
    fn test_split_util() {
        println!("{:?}", split_file_path("/"));
        println!("{:?}", split_file_path("/dev/"));
        println!("{:?}", split_file_path("/dev/proc/"));
        println!("{:?}", split_file_path("/a.txt"));
    }
}